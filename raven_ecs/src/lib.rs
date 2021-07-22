use std::mem;

type ID = usize;
type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Entity {
    id: ID,
    version: Version,
}

mod pool {
    use super::*;

    const PAGE_SIZE: usize = 100;

    /// A `Page` is either null or a pointer to an array of optional indices
    type Page = Option<Box<[Option<usize>; PAGE_SIZE]>>;

    struct Pool<T: 'static> {
        sparse: Vec<Page>,
        packed: Vec<ID>,
        components: Vec<T>,
    }

    impl<T: 'static> Pool<T> {
        fn new() -> Self {
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

        fn add(&mut self, entity_id: ID, component: T) {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // Is the vector large enough for us to be able to place the page at the correct index?
            if self.sparse.len() <= idx_to_page {
                // If not, make it bigger
                self.sparse.resize(idx_to_page + 1, None);
            }

            // Get a mut ref to the page (which is just a pointer to an array or null). If it is null, insert an empty page
            let mut page = self.sparse[idx_to_page].get_or_insert(Box::new([None; PAGE_SIZE]));

            // The new component will be pushed to the end of the packed arrays so that's the index that we should store
            page[idx_into_page] = Some(self.packed.len());

            self.packed.push(entity_id);
            self.components.push(component);
        }

        fn remove(&mut self, entity_id: ID) -> Option<T> {
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
            let mut page = self.sparse.get_mut(idx_to_page)?;

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

        fn get(&self, entity_id: ID) -> Option<&T> {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
            // equal to the length of the page, it would've been placed in the next page.
            let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

            Some(&self.components[packed_idx])
        }

        fn get_mut(&mut self, entity_id: ID) -> Option<&mut T> {
            let idx_to_page = Self::idx_to_page(entity_id);
            let idx_into_page = Self::idx_into_page(entity_id);

            // We don't need to check if `idx_into_page` is small enough because it always will. If it was greater or
            // equal to the length of the page, it would've been placed in the next page.
            let packed_idx = self.sparse.get(idx_to_page)?.as_ref()?[idx_into_page]?;

            Some(&mut self.components[packed_idx])
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn get() {
            let mut p: Pool<&'static str> = Pool::new();

            p.add(0, "A");
            assert_eq!(p.get(0), Some(&"A"));
        }

        #[test]
        fn get_mut() {
            let mut p: Pool<&'static str> = Pool::new();

            p.add(0, "A");
            *p.get_mut(0).unwrap() = "B";
            assert_eq!(p.get(0), Some(&"B"));
        }

        #[test]
        fn sparse_grows() {
            let mut p: Pool<&'static str> = Pool::new();

            assert_eq!(p.sparse.len(), 0);
            p.add(0, "A");
            assert_eq!(p.sparse.len(), 1);
            p.add(99, "B"); // Still in the first page
            assert_eq!(p.sparse.len(), 1);
            p.add(100, "C"); // Goes to the second page
            assert_eq!(p.sparse.len(), 2);
        }

        #[test]
        fn sparse_shrinks() {
            let mut p: Pool<&'static str> = Pool::new();

            p.add(0, "A");
            p.add(PAGE_SIZE - 1, "B");
            p.add(PAGE_SIZE, "C");

            assert_eq!(p.sparse.len(), 2);
            p.remove(0);
            assert_eq!(p.sparse.len(), 2);
            p.remove(PAGE_SIZE - 1);
            assert_eq!(p.sparse.len(), 2);
            p.remove(PAGE_SIZE);
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
            p.add(0, "A");
            assert_len_is(&p, 1);
            p.add(PAGE_SIZE - 1, "B");
            assert_len_is(&p, 2);
            p.add(PAGE_SIZE, "C");
            assert_len_is(&p, 3);
            p.remove(0);
            assert_len_is(&p, 2);
            p.remove(PAGE_SIZE - 1);
            assert_len_is(&p, 1);
            p.remove(PAGE_SIZE);
            assert_len_is(&p, 0);
        }

        #[test]
        fn remove_returns_component() {
            let mut p: Pool<&'static str> = Pool::new();

            p.add(0, "A");
            assert_eq!(p.remove(0), Some("A"));
        }

        #[test]
        fn remove_non_repeatable() {
            let mut p: Pool<&'static str> = Pool::new();

            p.add(0, "A");
            p.remove(0); // Should be Some("A")
            assert_eq!(p.remove(0), None);
        }

        use rand::{self, distributions::Uniform, distributions::Distribution, Rng};

        #[test]
        fn rand_io() {
            let mut rng = rand::thread_rng();
            let dist = Uniform::from(0..PAGE_SIZE);

            // Generate a random number of random entities and give each one a component (equal to the id +1, for simplicity)
            let mut entities: Vec<(ID, i32)> = Vec::new();
            for _ in 0..dist.sample(&mut rng) {
                let id: ID = dist.sample(&mut rng);
                entities.push((id, id as i32 + 1));
            }

            let mut p: Pool<i32> = Pool::new();
            for (entity_id, component) in entities {
                p.add(entity_id, component);
            }

            // Check with get
        }
    }
}
