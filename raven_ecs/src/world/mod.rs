use std::any::TypeId;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::{Component, Entity, ID, Version};
use crate::pool::{AnyPool, Pool};

pub mod query;
mod serde;

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
        self.entity_from_id(entity.id) == Some(entity)
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

    fn is_alive(&self, entity_id: ID) -> bool {
        matches!(self.entities.get(entity_id), Some(&(Some(stored_entity_id), _)) if stored_entity_id == entity_id)
    }

    fn entity_from_id(&self, entity_id: ID) -> Option<Entity> {
        if !self.is_alive(entity_id) {
            return None;
        }

        Some(Entity {
            id: entity_id,
            version: self.version_from_id(entity_id)?,
        })
    }

    fn version_from_id(&self, entity_id: ID) -> Option<Version> {
        if !self.is_alive(entity_id) {
            return None;
        }

        let &(_, version) = self.entities.get(entity_id)?;
        Some(version)
    }

    pub fn entities(&self) -> Vec<Entity> {
        let mut out = Vec::new();
        for entity_id in 0..self.entities.len() {
            if let Some(entity) = self.entity_from_id(entity_id) {
                out.push(entity);
            }
        }
        out
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;

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
        w.attach(e, CompX::new("A"));

        assert_eq!(w.get_one::<CompX>(e).as_deref(), Some(&CompX::new("A")));
    }

    #[test]
    fn different_components() {
        let mut w = World::default();

        let e = w.create();
        w.attach::<CompX>(e, CompX::new("A"));
        w.attach::<CompY>(e, CompY::new("B"));

        assert_eq!(w.get_one::<CompX>(e).as_deref(), Some(&CompX::new("A")));
        assert_eq!(w.get_one::<CompY>(e).as_deref(), Some(&CompY::new("B")));
    }

    #[test]
    fn detach_one() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, CompX::new("A"));
        w.attach(e, CompX::new("B"));
        w.detach_one::<CompX>(e);

        assert_eq!(deref_vec!(w.get_all::<CompX>(e)), vec![&CompX::new("B")]);
    }

    #[test]
    fn detach_all() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, CompX::new("A"));
        w.attach(e, CompX::new("B"));
        w.detach_all::<CompX>(e);

        assert_eq!(deref_vec!(w.get_all::<CompX>(e)), Vec::<&CompX>::new());
    }

    #[test]
    fn destroy_clears_components() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, CompX::new("A"));
        w.destroy(e);

        assert_eq!(w.get_one::<CompX>(e).as_deref(), None);
    }

    #[test]
    fn recycled_is_fresh() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach(e1, CompX::new("A"));
        w.destroy(e1);

        let e2 = w.create();

        assert_eq!(w.get_one::<CompX>(e2).as_deref(), None);
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
        w.attach(e, CompX::new("A"));

        {
            let mut comp = w.get_one_mut::<CompX>(e).unwrap();
            comp.f = comp.f.to_ascii_lowercase();
        }

        assert_eq!(w.get_one::<CompX>(e).as_deref(), Some(&CompX::new("a")));
    }

    #[test]
    fn get_all() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, CompX::new("A"));
        w.attach(e, CompX::new("B"));
        w.attach(e, CompX::new("C"));

        assert_eq!(deref_vec!(w.get_all::<CompX>(e)), vec![
            &CompX::new("A"),
            &CompX::new("B"),
            &CompX::new("C"),
        ]);
    }

    #[test]
    fn get_all_mut() {
        let mut w = World::default();

        let e = w.create();
        w.attach(e, CompX::new("A"));
        w.attach(e, CompX::new("B"));
        w.attach(e, CompX::new("C"));

        for mut n in w.get_all_mut::<CompX>(e) {
            n.f = n.f.to_ascii_lowercase();
        }

        assert_eq!(deref_vec!(w.get_all::<CompX>(e)), vec![
            &CompX::new("a"),
            &CompX::new("b"),
            &CompX::new("c"),
        ]);
    }

    #[test]
    fn entities() {
        let mut w = World::default();

        let e1 = w.create();
        let e2 = w.create();
        let e3 = w.create();
        let e4 = w.create();

        w.destroy(e3);

        assert_eq!(w.entities(), vec![
            e1,
            e2,
            e4,
        ]);
    }
}
