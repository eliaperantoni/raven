use std::mem;

type ID = usize;
type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Entity {
    id: ID,
    version: Version,
}

struct Pool<T: 'static> {
    sparse: Vec<Option<usize>>,
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

    fn add(&mut self, entity_id: ID, component: T) {
        if self.sparse.len() <= entity_id {
            self.sparse.resize(entity_id + 1, None);
        }
        self.sparse[entity_id] = Some(self.packed.len());

        self.packed.push(entity_id);
        self.components.push(component);
    }

    fn remove(&mut self, entity_id: ID) {
        let packed_idx = if let Some(packed_idx) = self.sparse[entity_id] {
            packed_idx
        } else {
            // This entity does not have a component. This is a NOOP
            return;
        };

        // This is how we remove the component from the packed entity array and the packed components list:
        //   1) Swap the last entity/component with the entity/component that we are deleting
        //   2) Pop from both arrays
        // But doing that, the last entity/component is swapped around and the corresponding index in the sparse array
        // is no longer valid.
        // This is where we fix that by settings its index to the place that it will take: the one of the component
        // that is going to be deleted
        self.sparse[*self.packed.last().unwrap()] = Some(packed_idx);

        let last_packed_idx = self.packed.len() - 1;

        self.packed.swap(packed_idx, last_packed_idx);
        self.packed.pop();

        self.components.swap(packed_idx, last_packed_idx);
        self.components.pop();

        self.sparse[entity_id] = None;
        while matches!(self.sparse.last(), Some(None)) {
            self.sparse.pop();
        }
    }

    fn get(&self, entity_id: ID) -> Option<&T> {
        self.sparse
            .get(entity_id)
            .copied()
            .and_then(move |packed_idx| Some(&self.components[packed_idx?]))
    }

    fn get_mut(&mut self, entity_id: ID) -> Option<&mut T> {
        self.sparse
            .get(entity_id)
            .copied()
            .and_then(move |packed_idx| Some(&mut self.components[packed_idx?]))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resizing() {
        let mut p: Pool<i32> = Pool::new();

        let sparse_size = |p: &Pool<_>| {
            p.sparse.len()
        };

        let packed_size = |p: &Pool<_>| {
            assert_eq!(p.packed.len(), p.components.len());
            p.packed.len()
        };

        assert_eq!(sparse_size(&p), 0);
        assert_eq!(packed_size(&p), 0);
        p.add(1, 1);
        assert_eq!(sparse_size(&p), 2);
        assert_eq!(packed_size(&p), 1);
        p.add(2, 2);
        assert_eq!(sparse_size(&p), 3);
        assert_eq!(packed_size(&p), 2);
        p.remove(1);
        assert_eq!(sparse_size(&p), 3);
        assert_eq!(packed_size(&p), 1);
        p.remove(2);
        assert_eq!(sparse_size(&p), 0);
        assert_eq!(packed_size(&p), 0);
    }
}
