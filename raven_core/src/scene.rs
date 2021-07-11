use std::collections::HashMap;

use crate::component::{Component, ComponentType};
use crate::entity::Entity;
use crate::ID;

pub struct Scene {
    components: HashMap<&'static str, Vec<>>
    entities: HashMap<ID, Entity>,
}

impl Scene {
    pub fn get_component<T: Component>(entity: ID) -> T {
        match T {}
    }
}
