use std::fmt::Formatter;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;

use crate::{Component, Entity, ID, Version};

use super::World;

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

#[derive(Deserialize)]
struct DeserializedEntity {
    id: ID,
    version: Version,
    components: Vec<Box<dyn Component>>,
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
                for component in pool.get_all_as_dyn(entity.id) {
                    se.components.push(Ref(component));
                }
            }

            state.serialize_element(&se)?;
        }
        state.end()
    }
}

struct WorldVisitor;

impl<'de> Visitor<'de> for WorldVisitor {
    type Value = World;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a world")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error> where S: SeqAccess<'de> {
        let mut world = World::default();

        while let Some(next) = seq.next_element::<DeserializedEntity>()? {
            let entity = Entity {
                id: next.id,
                version: next.version,
            };

            if world.entities.len() <= entity.id {
                world.entities.resize(entity.id + 1, (None, 0));
            }

            world.entities[entity.id] = (Some(entity.id), entity.version);

            for comp in next.components {
                comp.inject(&mut world, entity);
            }
        }

        {
            let destroyed_entities: Vec<ID> = world.entities
                .iter()
                .enumerate()
                .filter(|(entity_id, _)| !world.is_alive(*entity_id))
                .map(|(entity_id, _)| entity_id)
                .collect();

            let mut prev: Option<ID> = None;
            for entity_id in destroyed_entities {
                if let Some(prev) = prev {
                    world.entities[entity_id].0 = Some(prev);
                } else {
                    prev = Some(entity_id);
                }
            }

            world.destroyed_head = prev;
        }

        Ok(world)
    }
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(WorldVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;

    use super::*;

    #[test]
    fn serde() {
        let mut original = World::default();

        let e1 = original.create();
        let e2 = original.create();
        let e3 = original.create();
        let _e4 = original.create();

        original.destroy(e3);

        let e3 = original.create();

        original.attach(e1, CompX::new("A"));
        original.attach(e1, CompX::new("B"));
        original.attach(e1, CompY::new("C"));

        original.attach(e2, CompX::new("D"));
        original.attach(e2, CompY::new("E"));
        original.attach(e2, CompY::new("F"));

        original.attach(e3, CompY::new("G"));
        original.attach(e3, CompY::new("H"));

        let serialized = serde_json::to_string_pretty(&original).unwrap();

        let deserialized: World = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.entities(), deserialized.entities());

        for entity in original.entities() {
            assert_eq!(
                deref_vec!(original.get_all::<CompX>(entity)),
                deref_vec!(deserialized.get_all::<CompX>(entity)),
            );
            assert_eq!(
                deref_vec!(original.get_all::<CompY>(entity)),
                deref_vec!(deserialized.get_all::<CompY>(entity)),
            );
        }
    }
}
