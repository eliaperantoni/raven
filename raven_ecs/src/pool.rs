use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};

use smallvec::{SmallVec, smallvec};

use crate::{Component, ID};

const PAGE_SIZE: usize = 100;

/// A `Page` is either null or a pointer to an array of optional indices
type Page = Option<Box<[Option<usize>; PAGE_SIZE]>>;

type CompVec<T> = SmallVec<[RefCell<T>; 1]>;

pub struct Pool<T: Component> {
    sparse: Vec<Page>,
    packed: Vec<ID>,
    components: Vec<CompVec<T>>,
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
        // If this entity already has a component, then nothing should change regarding the sparse and packed arrays
        // and we can simply do an easy push
        if let Some(comp_vec) = self.get_comp_vec_mut(entity_id) {
            comp_vec.push(RefCell::new(component));
            return;
        }

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
        self.components.push(smallvec![RefCell::new(component)]);
    }

    pub fn detach_one(&mut self, entity_id: ID) -> Option<T> {
        // Bail out early if this entity doesn't have any component
        let comp_vec = self.get_comp_vec_mut(entity_id)?;

        if comp_vec.len() > 1 {
            // If this entity will be left with at least one component, then nothing will change regarding the sparse and
            // packed arrays. So we can just do a simple remove
            Some(comp_vec.remove(0).into_inner())
        } else {
            // Otherwise, this is the last component
            Some(self.detach_all(entity_id).remove(0))
        }
    }

    pub fn detach_all(&mut self, entity_id: ID) -> Vec<T> {
        let idx_to_page = Self::idx_to_page(entity_id);
        let idx_into_page = Self::idx_into_page(entity_id);

        // We will swap the last element of the packed arrays with the removed element. Get the packed_idx and the
        // entity_id of that last element
        let (packed_idx_of_last, entity_id_of_last) = {
            let len = self.packed.len();
            if len > 0 {
                (len - 1, self.packed[len - 1])
            } else {
                // We have no component at all, bail out
                return Vec::new();
            }
        };


        let (page, packed_idx) = match try {
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

            (page, packed_idx)
        } {
            Some(pair) => pair,
            None => return Vec::new()
        };

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
        self.components.pop().unwrap().into_vec().into_iter().map(|cell| cell.into_inner()).collect()
    }

    fn get_comp_vec(&self, entity_id: ID) -> Option<&CompVec<T>> {
        let idx_to_page = Self::idx_to_page(entity_id);
        let idx_into_page = Self::idx_into_page(entity_id);

        // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
        // equal to the length of the page, it would've been placed in the next page.
        let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

        Some(&self.components[packed_idx])
    }

    fn get_comp_vec_mut(&mut self, entity_id: ID) -> Option<&mut CompVec<T>> {
        let idx_to_page = Self::idx_to_page(entity_id);
        let idx_into_page = Self::idx_into_page(entity_id);

        // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
        // equal to the length of the page, it would've been placed in the next page.
        let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

        Some(&mut self.components[packed_idx])
    }

    pub fn count(&self, entity_id: ID) -> usize {
        let opt: Option<usize> = try {
            self.get_comp_vec(entity_id)?.len()
        };
        opt.unwrap_or(0)
    }

    pub fn get_one(&self, entity_id: ID) -> Option<Ref<T>> {
        self.get_nth(entity_id, 0)
    }

    pub fn get_one_mut(&self, entity_id: ID) -> Option<RefMut<T>> {
        self.get_nth_mut(entity_id, 0)
    }

    pub fn get_nth(&self, entity_id: ID, n: usize) -> Option<Ref<T>> {
        let comp_vec = self.get_comp_vec(entity_id)?;
        Some(comp_vec.get(n)?.borrow())
    }

    pub fn get_nth_mut(&self, entity_id: ID, n: usize) -> Option<RefMut<T>> {
        let comp_vec = self.get_comp_vec(entity_id)?;
        Some(comp_vec.get(n)?.borrow_mut())
    }

    pub fn get_all(&self, entity_id: ID) -> Vec<Ref<T>> {
        if let Some(comp_vec) = self.get_comp_vec(entity_id) {
            comp_vec.iter().map(|cell| cell.borrow()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_mut(&self, entity_id: ID) -> Vec<RefMut<T>> {
        if let Some(comp_vec) = self.get_comp_vec(entity_id) {
            comp_vec.iter().map(|cell| cell.borrow_mut()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn entities_ids(&self) -> Vec<ID> {
        self.packed.clone()
    }
}

pub trait AnyPool {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear_entity(&mut self, entity_id: ID);
    fn get_all_as_dyn(&self, entity_id: ID) -> Vec<Ref<dyn Component>>;
}

impl<T: Component> AnyPool for Pool<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clear_entity(&mut self, entity_id: ID) {
        self.detach_one(entity_id);
    }

    fn get_all_as_dyn(&self, entity_id: ID) -> Vec<Ref<dyn Component>> {
        self
            .get_all(entity_id)
            .into_iter()
            .map(|ref_| Ref::map(ref_, |comp| comp as &dyn Component))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::test::*;

    use super::*;

    #[test]
    fn count() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(0, CompX::new("B"));
        p.attach(0, CompX::new("C"));
        p.attach(0, CompX::new("D"));
        p.attach(0, CompX::new("E"));

        assert_eq!(p.count(0), 5);
    }

    #[test]
    fn get_nth() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(0, CompX::new("B"));

        assert_eq!(p.get_nth(0, 0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_nth(0, 1).as_deref(), Some(&CompX::new("B")));
        assert_eq!(p.get_nth(0, 2).as_deref(), None);
    }

    #[test]
    fn get_nth_mut() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(0, CompX::new("B"));

        *p.get_nth_mut(0, 1).unwrap() = CompX::new("Z");

        assert_eq!(p.get_nth(0, 0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_nth(0, 1).as_deref(), Some(&CompX::new("Z")));
        assert_eq!(p.get_nth(0, 2).as_deref(), None);
    }

    #[test]
    fn get_one() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("A")));
    }

    #[test]
    fn get_mut_mut() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        *p.get_one_mut(0).unwrap() = CompX::new("Z");
        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("Z")));
    }

    #[test]
    fn get_all() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(0, CompX::new("B"));
        p.attach(0, CompX::new("C"));

        assert_eq!(deref_vec!(p.get_all(0)), vec![
            &CompX::new("A"),
            &CompX::new("B"),
            &CompX::new("C"),
        ]);
    }

    #[test]
    fn get_all_mut() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(0, CompX::new("B"));
        p.attach(0, CompX::new("C"));

        for mut n in p.get_all_mut(0) {
            n.f = n.f.to_ascii_lowercase();
        }

        assert_eq!(deref_vec!(p.get_all(0)), vec![
            &CompX::new("a"),
            &CompX::new("b"),
            &CompX::new("c"),
        ]);
    }

    #[test]
    fn sparse_grows() {
        let mut p: Pool<CompX> = Pool::new();

        assert_eq!(p.sparse.len(), 0);
        p.attach(0, CompX::new("A"));
        assert_eq!(p.sparse.len(), 1);
        p.attach(99, CompX::new("B")); // Still in the first page
        assert_eq!(p.sparse.len(), 1);
        p.attach(100, CompX::new("C")); // Goes to the second page
        assert_eq!(p.sparse.len(), 2);
    }

    #[test]
    fn sparse_shrinks() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(PAGE_SIZE - 1, CompX::new("B"));
        p.attach(PAGE_SIZE, CompX::new("C"));

        assert_eq!(p.sparse.len(), 2);
        p.detach_one(0);
        assert_eq!(p.sparse.len(), 2);
        p.detach_one(PAGE_SIZE - 1);
        assert_eq!(p.sparse.len(), 2);
        p.detach_one(PAGE_SIZE);
        assert_eq!(p.sparse.len(), 0);
    }

    #[test]
    fn packed_arrays_len() {
        let mut p: Pool<CompX> = Pool::new();

        let assert_len_is = |p: &Pool<_>, len: usize| {
            assert_eq!(p.packed.len(), len);
            assert_eq!(p.components.len(), len);
        };

        assert_len_is(&p, 0);
        p.attach(0, CompX::new("A"));
        assert_len_is(&p, 1);
        p.attach(PAGE_SIZE - 1, CompX::new("B"));
        assert_len_is(&p, 2);
        p.attach(PAGE_SIZE, CompX::new("C"));
        assert_len_is(&p, 3);
        p.detach_one(0);
        assert_len_is(&p, 2);
        p.detach_one(PAGE_SIZE - 1);
        assert_len_is(&p, 1);
        p.detach_one(PAGE_SIZE);
        assert_len_is(&p, 0);
    }

    #[test]
    fn remove_returns_component() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("Z"));
        assert_eq!(p.detach_one(0), Some(CompX::new("Z")));
    }

    #[test]
    fn remove_non_repeatable() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("Z"));
        p.detach_one(0); // Should be Some(CompX {f: "Z"})
        assert_eq!(p.detach_one(0), None);
    }

    #[test]
    fn simple_add() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(1, CompX::new("B"));

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr[1] = Some(1); // Point to second element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![0, 1]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("A"))],
            smallvec![RefCell::new(CompX::new("B"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_one(1).as_deref(), Some(&CompX::new("B")));
    }

    #[test]
    fn add_not_adjacent() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(2, CompX::new("B"));

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr[2] = Some(1); // Point to second element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![0, 2]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("A"))],
            smallvec![RefCell::new(CompX::new("B"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_one(2).as_deref(), Some(&CompX::new("B")));
    }

    #[test]
    fn simple_remove_left() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(1, CompX::new("B"));

        p.detach_one(0);

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[1] = Some(0); // Point to first element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![1]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("B"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), None);
        assert_eq!(p.get_one(1).as_deref(), Some(&CompX::new("B")));
    }

    #[test]
    fn simple_remove_right() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(1, CompX::new("B"));

        p.detach_one(1);

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![0]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("A"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_one(1).as_deref(), None);
    }

    #[test]
    fn remove_not_adjacent_left() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(2, CompX::new("B"));

        p.detach_one(0);

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[2] = Some(0); // Point to first element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![2]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("B"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), None);
        assert_eq!(p.get_one(1).as_deref(), None);
        assert_eq!(p.get_one(2).as_deref(), Some(&CompX::new("B")));
    }

    #[test]
    fn remove_not_adjacent_right() {
        let mut p: Pool<CompX> = Pool::new();

        p.attach(0, CompX::new("A"));
        p.attach(2, CompX::new("B"));

        p.detach_one(2);

        assert_eq!(
            p.sparse,
            vec![Some(Box::new({
                let mut arr = [None; PAGE_SIZE];
                arr[0] = Some(0); // Point to first element in packed arrays
                arr
            }))]
        );

        assert_eq!(p.packed, vec![0]);

        let want_components: Vec<CompVec<_>> = vec![
            smallvec![RefCell::new(CompX::new("A"))],
        ];
        assert_eq!(p.components, want_components);

        assert_eq!(p.get_one(0).as_deref(), Some(&CompX::new("A")));
        assert_eq!(p.get_one(1).as_deref(), None);
        assert_eq!(p.get_one(2).as_deref(), None);
    }

    #[test]
    fn rand_io() {
        use rand;
        use rand::distributions::{Distribution, Uniform};
        use rand::seq::SliceRandom;
        use rand::Rng;

        const N_TARGET_ENTITIES_PER_PAGE: usize = PAGE_SIZE / 10;
        const N_TARGET_PAGES: usize = 10;

        let mut rng = rand::thread_rng();
        let dist = Uniform::from(0..PAGE_SIZE * N_TARGET_PAGES);

        // Generate a random number of random entities and give each one a component (equal to the id +1, for simplicity).
        // The bool tracks whether that entity should still be present in the pool (we will delete them).
        let mut entities: Vec<(ID, CompX, bool)> = Vec::new();
        for _ in 0..rng.gen_range(0..PAGE_SIZE * N_TARGET_ENTITIES_PER_PAGE) {
            // Find a new ID never used before
            let id: ID = loop {
                let new_id = dist.sample(&mut rng);
                if entities.iter().find(|(id, _, _)| *id == new_id).is_none() {
                    break new_id;
                }
            };

            entities.push((id, CompX::new(&id.to_string()), true));
        }

        let mut p: Pool<CompX> = Pool::new();
        for (entity_id, component, _) in entities.iter().cloned() {
            p.attach(entity_id, component);
        }

        entities.shuffle(&mut rng);

        loop {
            for (entity_id, component, alive) in entities.iter().cloned() {
                if alive {
                    assert_eq!(p.get_one(entity_id).as_deref(), Some(&component));
                } else {
                    assert_eq!(p.get_one(entity_id).as_deref(), None);
                }
            }

            // Delete the first entity still alive
            if let Some((entity_id, _, alive)) = entities.iter_mut().find(|(_, _, alive)| *alive) {
                *alive = false;
                p.detach_one(*entity_id);
            } else {
                break;
            }
        }

        assert_eq!(p.sparse.len(), 0);
        assert_eq!(p.packed.len(), 0);
        assert_eq!(p.components.len(), 0);
    }
}
