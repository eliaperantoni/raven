use std::any::TypeId;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::{Component, Entity, ID, Version};
use crate::pool::{AnyPool, Pool};

pub mod query;

pub struct World {
    entities: Vec<(Option<ID>, Version)>,
    destroyed_head: Option<usize>,

    pools: HashMap<TypeId, Box<dyn AnyPool>>,
}

impl Default for World {
    fn default() -> Self {
        World {
            entities: Vec::new(),
            destroyed_head: None,
            pools: HashMap::new(),
        }
    }
}

impl World {
    pub fn create(&mut self) -> Entity {
        if let Some(destroyed_next) = self.destroyed_head {
            // Move destroyed_head to the next destroyed entity (or None)
            self.destroyed_head = self.entities[destroyed_next].0;

            let (entity_id, version) = &mut self.entities[destroyed_next];

            // Set entity id to its own index
            *entity_id = Some(destroyed_next);

            Entity {
                id: destroyed_next,
                version: *version,
            }
        } else {
            let entity = Entity {
                id: self.entities.len(),
                version: 0,
            };

            self.entities.push((Some(entity.id), entity.version));

            entity
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        if !self.entity_exists(entity) {
            return;
        }

        // Remove all components from this entity
        for pool in self.pools.values_mut() {
            pool.clear_entity(entity.id);
        }

        let (entity_id, version) = self.entities.get_mut(entity.id).unwrap();

        // Bump version
        *version += 1;

        // Link to what destroyed_head is currently pointing at
        *entity_id = self.destroyed_head;

        // Point destroyed_head to newly destroyed entity
        self.destroyed_head = Some(entity.id);
    }

    fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.get(entity.id) == Some(&(Some(entity.id), entity.version))
    }

    fn pool<T: Component>(&self) -> Option<&Pool<T>> {
        let t_id = TypeId::of::<T>();

        let p = self.pools.get(&t_id)?;
        Some(p.as_any().downcast_ref::<Pool<T>>().unwrap())
    }

    fn pool_mut<T: Component>(&mut self) -> Option<&mut Pool<T>> {
        let t_id = TypeId::of::<T>();

        let p = self.pools.get_mut(&t_id)?;
        Some(p.as_any_mut().downcast_mut::<Pool<T>>().unwrap())
    }

    pub fn attach<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.entity_exists(entity) {
            return;
        }

        let t_id = TypeId::of::<T>();

        // Does a pool for this component T exist already? If not, create an empty one
        self.pools.entry(t_id).or_insert_with(|| Box::new(Pool::<T>::new()));

        let pool = self.pool_mut::<T>().unwrap();
        pool.attach(entity.id, component);
    }

    pub fn detach_one<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.entity_exists(entity) {
            return None;
        }

        let pool = self.pool_mut::<T>()?;
        pool.detach_one(entity.id)
    }

    pub fn detach_all<T: Component>(&mut self, entity: Entity) -> Vec<T> {
        if !self.entity_exists(entity) {
            return Vec::new();
        }

        if let Some(pool) = self.pool_mut::<T>() {
            pool.detach_all(entity.id)
        } else {
            Vec::new()
        }
    }

    pub fn get_one<T: Component>(&self, entity: Entity) -> Option<impl Deref<Target=T> + '_> {
        if !self.entity_exists(entity) {
            return None;
        }

        let p = self.pool::<T>()?;
        p.get_one(entity.id)
    }

    pub fn get_one_mut<T: Component>(&mut self, entity: Entity) -> Option<impl DerefMut<Target=T> + '_> {
        if !self.entity_exists(entity) {
            return None;
        }

        let p = self.pool::<T>()?;
        p.get_one_mut(entity.id)
    }

    pub fn get_all<T: Component>(&self, entity: Entity) -> Vec<impl Deref<Target=T> + '_> {
        if !self.entity_exists(entity) {
            return Vec::new();
        }

        let p = if let Some(p) = self.pool::<T>() {
            p
        } else {
            return Vec::new();
        };

        p.get_all(entity.id)
    }

    pub fn get_all_mut<T: Component>(&mut self, entity: Entity) -> Vec<impl DerefMut<Target=T> + '_> {
        if !self.entity_exists(entity) {
            return Vec::new();
        }

        let p = if let Some(p) = self.pool::<T>() {
            p
        } else {
            return Vec::new();
        };

        p.get_all_mut(entity.id)
    }

    fn entity_from_id(&self, entity_id: ID) -> Option<Entity> {
        let &(_, version) = self.entities.get(entity_id)?;
        Some(Entity {
            id: entity_id,
            version,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_entity() {
        let mut w = World::default();

        assert_eq!(w.create(), Entity { id: 0, version: 0 });
        assert_eq!(w.create(), Entity { id: 1, version: 0 });
    }

    #[test]
    fn recycle() {
        let mut w = World::default();

        let e = w.create();
        w.destroy(e);
        assert_eq!(w.create(), Entity { id: 0, version: 1 });
    }

    #[test]
    fn attach() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 1);

        assert_eq!(w.get_one::<i32>(e).as_deref(), Some(&1));
    }

    #[test]
    fn different_components() {
        let mut w = World::default();

        let e = w.create();
        w.attach::<char>(e, 'A');
        w.attach::<i32>(e, 10);

        assert_eq!(w.get_one::<char>(e).as_deref(), Some(&'A'));
        assert_eq!(w.get_one::<i32>(e).as_deref(), Some(&10));
    }

    #[test]
    fn detach_one() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 'A');
        w.attach(e, 'B');
        w.detach_one::<char>(e);

        assert_eq!(deref_vec!(w.get_all::<char>(e)), vec![&'B']);
    }

    #[test]
    fn detach_all() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 'A');
        w.attach(e, 'B');
        w.detach_all::<char>(e);

        assert_eq!(deref_vec!(w.get_all::<char>(e)), Vec::<&char>::new());
    }

    #[test]
    fn destroy_clears_components() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 'A');
        w.destroy(e);

        assert_eq!(w.get_one::<char>(e).as_deref(), None);
    }

    #[test]
    fn recycled_is_fresh() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach(e1, 'A');
        w.destroy(e1);

        let e2 = w.create();

        assert_eq!(w.get_one::<char>(e2).as_deref(), None);
    }

    #[test]
    fn longer_destroyed_list() {
        let mut w = World::default();

        let entities: Vec<_> = (0..10).map(|_| w.create()).collect();

        for entity in entities {
            w.destroy(entity);
        }

        assert_eq!(w.create(), Entity { id: 9, version: 1 });
        assert_eq!(w.create(), Entity { id: 8, version: 1 });
    }

    #[test]
    fn mutability() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 'A');

        *w.get_one_mut::<char>(e).unwrap() = 'B';

        assert_eq!(w.get_one::<char>(e).as_deref(), Some(&'B'));
    }

    #[test]
    fn get_all() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 'A');
        w.attach(e, 'B');
        w.attach(e, 'C');

        assert_eq!(deref_vec!(w.get_all::<char>(e)), vec![&'A', &'B', &'C']);
    }

    #[test]
    fn get_all_mut() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, 1);
        w.attach(e, 2);
        w.attach(e, 3);

        for mut n in w.get_all_mut::<i32>(e) {
            *n *= 10;
        }

        assert_eq!(deref_vec!(w.get_all::<i32>(e)), vec![&10, &20, &30]);
    }
}
