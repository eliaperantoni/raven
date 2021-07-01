use crate::component::CameraComponent;
use crate::entity::Entity;

use super::System;

pub struct CameraSystem {}

impl Default for CameraSystem {
    fn default() -> Self {
        CameraSystem {}
    }
}

impl System for CameraSystem {
    fn visit_entity(&mut self, entity: &mut Entity) {
        if let Some(camera) = entity.get_component::<CameraComponent>() {
            println!("Found camera! FOV: {}", camera.fov);
        }
    }
}
