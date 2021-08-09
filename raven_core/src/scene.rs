use raven_ecs::world::World;

use std::collections::HashMap;

use crate::component::{Component, ComponentType};
use crate::entity::Entity;
use crate::ID;

pub struct Scene {
    world: World,
}

impl Scene {
    pub fn get_component<T: Component>(entity: ID) -> T {
        match T {}
    }
}
