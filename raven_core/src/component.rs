use std::path::PathBuf;

use glam::Mat4;
use serde::{Deserialize, Serialize};

use raven_ecs::Entity;

#[derive(Serialize, Deserialize, Clone)]
struct TransformComponent(Mat4);

#[derive(Serialize, Deserialize, Clone)]
struct HierarchyComponent {
    parent: Option<Entity>,
    children: Vec<Entity>,
}

#[derive(Serialize, Deserialize, Clone)]
struct MeshComponent {
    mesh: Option<PathBuf>,
    mat: Option<PathBuf>,
}
