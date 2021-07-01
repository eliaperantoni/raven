use glam::{Mat4, Vec3};

use crate::component::CameraComponent;
use crate::entity::Entity;

use super::System;

pub struct CameraSystem {
    cam_pos: Vec3,
    cam_forward: Vec3,
}

impl Default for CameraSystem {
    fn default() -> Self {
        CameraSystem {
            cam_pos: Vec3::default(),
            cam_forward: Vec3::default(),
        }
    }
}

impl System for CameraSystem {
    fn visit_entity(&mut self, entity: &mut Entity) {
        if let Some(camera) = entity.get_component::<CameraComponent>() {
            println!("Found camera! FOV: {}", camera.fov);
            self.cam_pos = entity.transform.position;
            todo!()
        }
    }
}
