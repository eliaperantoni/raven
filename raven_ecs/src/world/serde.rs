use std::any::TypeId;
use std::collections::HashMap;

use serde::{Serialize, Serializer, Deserialize};
use serde::ser::SerializeSeq;

use crate::{ID, Version};
use crate::pool::AnyPool;

use super::World;

#[derive(Serialize, Deserialize)]
struct SerializedEntity {
    id: ID,
    version: Version,
    // components: HashMap<TypeId, Box<dyn AnyPool>>,
}

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let entities = self.entities();

        let mut state = serializer.serialize_seq(Some(entities.len()))?;
        for entity in entities {
            let se = SerializedEntity {
                id: entity.id,
                version: entity.version,
            };

            state.serialize_element(&se);
        }
        state.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde() {
        let mut w = World::default();

        let e1 = w.create();
        let e2 = w.create();
        let e3 = w.create();
        let e4 = w.create();

        w.destroy(e3);

        let e3 = w.create();

        w.attach(e1, 1);
        w.attach(e1, 2);
        w.attach(e1, 'a');

        w.attach(e2, 3);
        w.attach(e2, 'b');
        w.attach(e2, 'c');

        w.attach(e3, 'd');
        w.attach(e3, 'e');

        println!("{}", serde_json::to_string(&w).unwrap());
    }
}
