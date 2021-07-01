use glam::{EulerRot, Mat4, Vec3};

use crate::component::CameraComponent;
use crate::entity::Entity;

use super::System;

#[derive(Debug)]
pub struct CameraSystem {
    cam_pos: Vec3,
    cam_target: Vec3,
    cam_up: Vec3,
    cam_fov: f32,

    pub aspect_ratio: f32,
}

impl CameraSystem {
    pub fn get_view_mat(&self) -> Mat4 {
        Mat4::look_at_rh(self.cam_pos, self.cam_target, self.cam_up)
    }

    pub fn get_proj_mat(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.cam_fov, self.aspect_ratio, 0.1, 100.0)
    }
}

impl Default for CameraSystem {
    fn default() -> Self {
        CameraSystem {
            cam_pos: Vec3::default(),
            cam_target: Vec3::default(),
            cam_up: Vec3::default(),
            cam_fov: 90.0,

            aspect_ratio: 1.0,
        }
    }
}

impl System for CameraSystem {
    fn visit_entity(&mut self, entity: &mut Entity) {
        if let Some(camera) = entity.get_component::<CameraComponent>() {
            self.cam_pos = entity.transform.position;

            let forward = {
                let (x, y, z) = entity.transform.rotation.to_euler(EulerRot::XYZ);
                Vec3::new(x, y, z).normalize()
            };

            self.cam_target = self.cam_pos + forward;

            let right = Vec3::cross(forward, Vec3::Y).normalize();

            self.cam_up = Vec3::cross(right, forward).normalize();

            self.cam_fov = camera.fov;
        }
    }
}
