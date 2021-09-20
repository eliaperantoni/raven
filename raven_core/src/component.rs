use std::path::PathBuf;

use glam::Mat4;
use serde::{Deserialize, Serialize};

use raven_ecs::{Component, Entity};

use crate::resource::{Scene, Texture};
use crate::vao::Vao;

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct TransformComponent(pub Mat4);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NameComponent(pub String);

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
    #[serde(skip)]
    pub(crate) tex: Option<Texture>,
}

impl MeshComponent {
    pub fn new(mesh_path: PathBuf, mat_path: PathBuf) -> MeshComponent {
        MeshComponent {
            mesh: mesh_path,
            mat: mat_path,

            vao: None,
            tex: None,
        }
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct CameraComponent {}

#[derive(Component, Serialize, Deserialize)]
pub struct SceneComponent {
    pub scene: PathBuf,
    #[serde(skip)]
    pub(crate) loaded: Option<Scene>,
}
