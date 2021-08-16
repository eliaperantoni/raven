use std::any::TypeId;
use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::SerializeSeq;

use crate::{Component, ID, Version};
use crate::pool::AnyPool;

use super::World;
use std::ops::Deref;
use serde::de::Error;

// Serde is dumb and doesn't impl Serialize for std::cell::Ref. We'll do it ourselves
struct Ref<'a, T: ?Sized>(std::cell::Ref<'a, T>);

impl<'a, T: ?Sized + Serialize> Serialize for Ref<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.0.deref().serialize(serializer)
    }
}

#[derive(Serialize)]
struct SerializedEntity<'a> {
    id: ID,
    version: Version,
    components: Vec<Ref<'a, dyn Component>>,
}

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let entities = self.entities();

        let mut state = serializer.serialize_seq(Some(entities.len()))?;
        for entity in entities {
            let mut se = SerializedEntity {
                id: entity.id,
                version: entity.version,
                components: Vec::new(),
            };

            for pool in self.pools.values() {
                for component in pool.get_all_as_any(entity.id) {
                    se.components.push(Ref(component));
                }
            }

            state.serialize_element(&se)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;

    use super::*;

    #[test]
    fn serde() {
        let mut w = World::default();

        let e1 = w.create();
        let e2 = w.create();
        let e3 = w.create();
        let _e4 = w.create();

        w.destroy(e3);

        let e3 = w.create();

        w.attach(e1, CompX::new("A"));
        w.attach(e1, CompX::new("B"));
        w.attach(e1, CompY::new("C"));

        w.attach(e2, CompX::new("D"));
        w.attach(e2, CompY::new("E"));
        w.attach(e2, CompY::new("F"));

        w.attach(e3, CompY::new("G"));
        w.attach(e3, CompY::new("H"));

        println!("{}", serde_json::to_string_pretty(&w).unwrap());
    }
}
