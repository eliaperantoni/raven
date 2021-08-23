use std::path::PathBuf;

use glam::Mat4;
use serde::{Deserialize, Serialize};

use raven_ecs::{Entity, Component};
use crate::vao::Vao;

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct TransformComponent(pub Mat4);

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct HierarchyComponent {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

#[derive(Component, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh: PathBuf,
    pub mat: PathBuf,
    #[serde(skip)]
    pub(crate) vao: Option<Vao>,
}

impl MeshComponent {
    pub fn new(mesh_path: PathBuf, mat_path: PathBuf) -> MeshComponent {
        MeshComponent {
            mesh: mesh_path,
            mat: mat_path,
            vao: None,
        }
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
}
