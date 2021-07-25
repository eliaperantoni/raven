use std::any::Any;
use std::iter::empty;

type ID = usize;
type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Entity {
    id: ID,
    version: Version,
}

pub trait Component: 'static + Sized {}

impl<T: 'static> Component for T {}

mod pool {
    use super::*;

    const PAGE_SIZE: usize = 100;

    /// A `Page` is either null or a pointer to an array of optional indices
    type Page = Option<Box<[Option<usize>; PAGE_SIZE]>>;

    pub struct Pool<T: Component> {
        sparse: Vec<Page>,
        packed: Vec<ID>,
        components: Vec<T>,
    }

    impl<T: Component> Pool<T> {
        pub fn new() -> Self {
            Self {
                sparse: Vec::new(),
                packed: Vec::new(),
                components: Vec::new(),
            }
        }

        /// Index of the page where the entity with the given ID should reside
        fn idx_to_page(entity_id: ID) -> usize {
            entity_id / PAGE_SIZE
        }

        /// Index into the page where the entity with the given ID should reside
        fn idx_into_page(entity_id: ID) -> usize {
            entity_id % PAGE_SIZE
        }

        pub fn attach(&mut self, entity_id: ID, component: T) {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // Is the vector large enough for us to be able to place the page at the correct index?
            if self.sparse.len() <= idx_to_page {
                // If not, make it bigger
                self.sparse.resize(idx_to_page + 1, None);
            }

            // Get a mut ref to the page (which is just a pointer to an array or null). If it is null, insert an empty page
            let page = self.sparse[idx_to_page].get_or_insert(Box::new([None; PAGE_SIZE]));

            // The new component will be pushed to the end of the packed arrays so that's the index that we should store
            page[idx_into_page] = Some(self.packed.len());

            self.packed.push(entity_id);
            self.components.push(component);
        }

        pub fn detach(&mut self, entity_id: ID) -> Option<T> {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // We will swap the last element of the packed arrays with the removed element. Get the packed_idx and the
            // entity_id of that last element
            let (packed_idx_of_last, entity_id_of_last) = {
                let len = self.packed.len();
                if len > 0 {
                    (len - 1, self.packed[len - 1])
                } else {
                    // We have no component at all
                    return None;
                }
            };

            // Get a mut ref to the page. If the array is too small for the page to be there, then this entity does not
            // have a component in this pool and we can return.
            let page = self.sparse.get_mut(idx_to_page)?;

            // Get a mut ref to the inner array. If the page is `None`, then this entity does not have a component in this
            // pool and we can return.
            let page = page.as_mut()?;

            // Index the page to get the index into the packed arrays.
            // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
            // equal to the length of the page, it would've been placed in the next page.
            // If the element was `None`, then this entity does not have a component in this pool and we can return.
            let packed_idx = page[idx_into_page]?;

            // Remove the element from the page, we read the packed index so we don't need that anymore
            page[idx_into_page] = None;

            // If the page is now empty, delete it
            if page.iter().all(|e| e.is_none()) {
                self.sparse[idx_to_page] = None;

                // If the sparse array need not to be so large anymore because the page got deleted, shrink it
                while matches!(self.sparse.last(), Some(None)) {
                    self.sparse.pop();
                }
            }

            // Adjust the packed index for the last element of the packed arrays since we will be swapping it with the
            // deleted element. But only if the element we're deleting and the last one are different. Otherwise we
            // may have already deleted the page that we wish to now update.
            if entity_id != entity_id_of_last {
                let idx_to_page_of_last = Self::idx_to_page(entity_id_of_last);
                let idx_into_page_of_last = Self::idx_into_page(entity_id_of_last);

                self.sparse[idx_to_page_of_last].as_mut().unwrap()[idx_into_page_of_last] = Some(packed_idx);
            }

            // Swap the last element with the packed index in both packed arrays
            self.packed.swap(packed_idx, packed_idx_of_last);
            self.components.swap(packed_idx, packed_idx_of_last);

            // Resize both packed arrays
            self.packed.pop();
            Some(self.components.pop().unwrap())
        }

        pub fn get(&self, entity_id: ID) -> Option<&T> {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
            // equal to the length of the page, it would've been placed in the next page.
            let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

            Some(&self.components[packed_idx])
        }

        pub fn get_mut(&mut self, entity_id: ID) -> Option<&mut T> {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
            // equal to the length of the page, it would've been placed in the next page.
            let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

            Some(&mut self.components[packed_idx])
        }

        pub fn iter(&self) -> impl ExactSizeIterator<Item=(&ID, &T)> {
            self.packed.iter().zip(self.components.iter())
        }

        pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item=(&ID, &mut T)> {
            self.packed.iter().zip(self.components.iter_mut())
        }
    }

    pub trait AnyPool {
        fn as_any(&self) -> &dyn Any;
        fn as_any_mut(&mut self) -> &mut dyn Any;
        fn clear_entity(&mut self, entity_id: ID);
    }

    impl<T: Component> AnyPool for Pool<T> {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clear_entity(&mut self, entity_id: ID) {
            self.detach(entity_id);
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn iter() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(1, "B");
            p.attach(2, "C");

            let mut it = p.iter();
            assert_eq!(it.next(), Some((&0, &"A")));
            assert_eq!(it.next(), Some((&1, &"B")));
            assert_eq!(it.next(), Some((&2, &"C")));
            assert_eq!(it.next(), None);
        }

        #[test]
        fn iter_mut() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(1, "B");
            p.attach(2, "C");

            {
                let mut it = p.iter_mut();

                let (_, component) = it.next().unwrap();
                *component = "Z";
            }

            let mut it = p.iter();
            assert_eq!(it.next(), Some((&0, &"Z")));
            assert_eq!(it.next(), Some((&1, &"B")));
            assert_eq!(it.next(), Some((&2, &"C")));
            assert_eq!(it.next(), None);
        }

        #[test]
        fn get() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            assert_eq!(p.get(0), Some(&"A"));
        }

        #[test]
        fn get_mut() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            *p.get_mut(0).unwrap() = "B";
            assert_eq!(p.get(0), Some(&"B"));
        }

        #[test]
        fn sparse_grows() {
            let mut p: Pool<&'static str> = Pool::new();

            assert_eq!(p.sparse.len(), 0);
            p.attach(0, "A");
            assert_eq!(p.sparse.len(), 1);
            p.attach(99, "B"); // Still in the first page
            assert_eq!(p.sparse.len(), 1);
            p.attach(100, "C"); // Goes to the second page
            assert_eq!(p.sparse.len(), 2);
        }

        #[test]
        fn sparse_shrinks() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(PAGE_SIZE - 1, "B");
            p.attach(PAGE_SIZE, "C");

            assert_eq!(p.sparse.len(), 2);
            p.detach(0);
            assert_eq!(p.sparse.len(), 2);
            p.detach(PAGE_SIZE - 1);
            assert_eq!(p.sparse.len(), 2);
            p.detach(PAGE_SIZE);
            assert_eq!(p.sparse.len(), 0);
        }

        #[test]
        fn packed_arrays_len() {
            let mut p: Pool<&'static str> = Pool::new();

            let assert_len_is = |p: &Pool<_>, len: usize| {
                assert_eq!(p.packed.len(), len);
                assert_eq!(p.components.len(), len);
            };

            assert_len_is(&p, 0);
            p.attach(0, "A");
            assert_len_is(&p, 1);
            p.attach(PAGE_SIZE - 1, "B");
            assert_len_is(&p, 2);
            p.attach(PAGE_SIZE, "C");
            assert_len_is(&p, 3);
            p.detach(0);
            assert_len_is(&p, 2);
            p.detach(PAGE_SIZE - 1);
            assert_len_is(&p, 1);
            p.detach(PAGE_SIZE);
            assert_len_is(&p, 0);
        }

        #[test]
        fn remove_returns_component() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            assert_eq!(p.detach(0), Some("A"));
        }

        #[test]
        fn remove_non_repeatable() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.detach(0); // Should be Some("A")
            assert_eq!(p.detach(0), None);
        }

        #[test]
        fn simple_add() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(1, "B");

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr[1] = Some(1); // Point to second element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![0, 1]);
            assert_eq!(p.components, vec!["A", "B"]);

            assert_eq!(p.get(0), Some(&"A"));
            assert_eq!(p.get(1), Some(&"B"));
        }

        #[test]
        fn add_not_adjacent() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(2, "B");

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr[2] = Some(1); // Point to second element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![0, 2]);
            assert_eq!(p.components, vec!["A", "B"]);

            assert_eq!(p.get(0), Some(&"A"));
            assert_eq!(p.get(2), Some(&"B"));
        }

        #[test]
        fn simple_remove_left() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(1, "B");

            p.detach(0);

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[1] = Some(0); // Point to first element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![1]);
            assert_eq!(p.components, vec!["B"]);

            assert_eq!(p.get(0), None);
            assert_eq!(p.get(1), Some(&"B"));
        }

        #[test]
        fn simple_remove_right() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(1, "B");

            p.detach(1);

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![0]);
            assert_eq!(p.components, vec!["A"]);

            assert_eq!(p.get(0), Some(&"A"));
            assert_eq!(p.get(1), None);
        }

        #[test]
        fn remove_not_adjacent_left() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(2, "B");

            p.detach(0);

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[2] = Some(0); // Point to first element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![2]);
            assert_eq!(p.components, vec!["B"]);

            assert_eq!(p.get(0), None);
            assert_eq!(p.get(1), None);
            assert_eq!(p.get(2), Some(&"B"));
        }

        #[test]
        fn remove_not_adjacent_right() {
            let mut p: Pool<&'static str> = Pool::new();

            p.attach(0, "A");
            p.attach(2, "B");

            p.detach(2);

            assert_eq!(p.sparse, vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr
            }))]);

            assert_eq!(p.packed, vec![0]);
            assert_eq!(p.components, vec!["A"]);

            assert_eq!(p.get(0), Some(&"A"));
            assert_eq!(p.get(1), None);
            assert_eq!(p.get(2), None);
        }

        #[test]
        fn rand_io() {
            use rand;
            use rand::Rng;
            use rand::distributions::{Distribution, Uniform};
            use rand::seq::SliceRandom;

            const N_TARGET_ENTITIES_PER_PAGE: usize = PAGE_SIZE / 10;
            const N_TARGET_PAGES: usize = 10;

            let mut rng = rand::thread_rng();
            let dist = Uniform::from(0..PAGE_SIZE * N_TARGET_PAGES);

            // Generate a random number of random entities and give each one a component (equal to the id +1, for simplicity).
            // The bool tracks whether that entity should still be present in the pool (we will delete them).
            let mut entities: Vec<(ID, i32, bool)> = Vec::new();
            for _ in 0..rng.gen_range(0..PAGE_SIZE * N_TARGET_ENTITIES_PER_PAGE) {
                // Find a new ID never used before
                let id: ID = loop {
                    let new_id = dist.sample(&mut rng);
                    if entities.iter().find(|(id, _, _)| *id == new_id).is_none() {
                        break new_id;
                    }
                };

                entities.push((id, id as i32 + 1, true));
            }

            let mut p: Pool<i32> = Pool::new();
            for (entity_id, component, _) in entities.iter().copied() {
                p.attach(entity_id, component);
            }

            entities.shuffle(&mut rng);

            loop {
                for (entity_id, component, alive) in entities.iter().copied() {
                    if alive {
                        assert_eq!(p.get(entity_id), Some(&component));
                    } else {
                        assert_eq!(p.get(entity_id), None);
                    }
                }

                // Delete the first entity still alive
                if let Some((entity_id, _, alive)) = entities.iter_mut().find(|(_, _, alive)| *alive) {
                    *alive = false;
                    p.detach(*entity_id);
                } else {
                    break;
                }
            }

            assert_eq!(p.sparse.len(), 0);
            assert_eq!(p.packed.len(), 0);
            assert_eq!(p.components.len(), 0);
        }
    }
}

use pool::{Pool, AnyPool};

use std::any::TypeId;
use std::collections::HashMap;

struct World {
    entities: Vec<(Option<ID>, Version)>,
    destroyed_head: Option<usize>,

    pools: HashMap<TypeId, Box<dyn AnyPool>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
            destroyed_head: None,
            pools: HashMap::new(),
        }
    }

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
        if self.entities.get(entity.id) != Some(&(Some(entity.id), entity.version)) {
            // Entity that should be deleted never existed or got recycled or is destroyed, return and this is a NOOP
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
        if let Some((entity_id, version)) = self.entities.get(entity.id) {
            *entity_id == Some(entity.id) && *version == entity.version
        } else {
            false
        }
    }

    fn pool<T: Component>(&self) -> Option<&Pool<T>> {
        let t_id = TypeId::of::<T>();

        self.pools.get(&t_id)?.as_any().downcast_ref::<Pool<T>>()
    }

    fn pool_mut<T: Component>(&mut self) -> Option<&mut Pool<T>> {
        let t_id = TypeId::of::<T>();

        self.pools.get_mut(&t_id)?.as_any_mut().downcast_mut::<Pool<T>>()
    }

    pub fn attach<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.entity_exists(entity) {
            return
        }

        let t_id = TypeId::of::<T>();

        // Does a pool for this component T exist already? If not, create an empty one
        if !self.pools.contains_key(&t_id) {
            self.pools.insert(t_id, Box::new(Pool::<T>::new()));
        }

        let mut pool = self.pool_mut::<T>().unwrap();
        pool.attach(entity.id, component);
    }

    pub fn detach<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.entity_exists(entity) {
            return None;
        }

        let mut pool = self.pool_mut::<T>()?;
        pool.detach(entity.id)
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entity_exists(entity) {
            return None;
        }

        let pool = self.pool::<T>()?;
        pool.get(entity.id)
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entity_exists(entity) {
            return None;
        }

        let pool = self.pool_mut::<T>()?;
        pool.get_mut(entity.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_entity() {
        let mut w = World::new();

        assert_eq!(w.create(), Entity { id: 0, version: 0 });
        assert_eq!(w.create(), Entity { id: 1, version: 0 });
    }

    #[test]
    fn recycle() {
        let mut w = World::new();

        let e = w.create();
        w.destroy(e);
        assert_eq!(w.create(), Entity { id: 0, version: 1 });
    }

    #[test]
    fn attach() {
        let mut w = World::new();

        let e = w.create();
        w.attach(e, "A");

        assert_eq!(w.get_component::<&'static str>(e), Some(&"A"));
    }

    #[test]
    fn different_components() {
        let mut w = World::new();

        let e = w.create();
        w.attach::<&'static str>(e, "A");
        w.attach::<i32>(e, 10);

        assert_eq!(w.get_component::<&'static str>(e), Some(&"A"));
        assert_eq!(w.get_component::<i32>(e), Some(&10));
    }

    #[test]
    fn detach() {
        let mut w = World::new();

        let e = w.create();
        w.attach(e, "A");
        w.detach::<&'static str>(e);

        assert_eq!(w.get_component::<&'static str>(e), None);
    }

    #[test]
    fn destroy_clears_components() {
        let mut w = World::new();

        let e = w.create();
        w.attach(e, "A");
        w.destroy(e);

        assert_eq!(w.get_component::<&'static str>(e), None);
    }

    #[test]
    fn recycled_is_fresh() {
        let mut w = World::new();

        let e1 = w.create();
        w.attach(e1, "A");
        w.destroy(e1);

        let e2 = w.create();

        assert_eq!(w.get_component::<&'static str>(e2), None);
    }

    #[test]
    fn longer_destroyed_list() {
        let mut w = World::new();

        let entities: Vec<_> = (0..10).map(|_| w.create()).collect();

        for entity in entities {
            w.destroy(entity);
        }

        assert_eq!(w.create(), Entity{ id: 9, version: 1 });
        assert_eq!(w.create(), Entity{ id: 8, version: 1 });
    }
}

#[cfg(test)]
mod test_query {
    use super::*;

    #[test]
    fn query() {
        let mut w = World::new();

        let e = w.create();
        w.attach::<i32>(e, 10);
        w.attach::<&'static str>(e, "A");

    }
}
