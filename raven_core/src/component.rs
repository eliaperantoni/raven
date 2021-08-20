use std::path::PathBuf;

use glam::Mat4;
use serde::{Deserialize, Serialize};

use raven_ecs::{Entity, Component};

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct TransformComponent(pub Mat4);

#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct HierarchyComponent {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MeshComponent {
    pub mesh: Option<PathBuf>,
    pub mat: Option<PathBuf>,
}
