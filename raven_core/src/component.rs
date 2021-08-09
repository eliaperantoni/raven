use std::any::{Any, TypeId};

use glam::Mat4;
use raven_ecs::Entity;

use crate::ResourceID;
use crate::material::Material;
use crate::mesh::Mesh;

#[derive(Debug)]
pub struct TransformComponent {
    transform: Mat4,
}

#[derive(Debug)]
pub struct HierarchyComponent {
    parent: Option<Entity>,
    children: Vec<Entity>,
}

#[derive(Debug)]
pub struct MeshComponent {
    mesh: ResourceID,
    mat: ResourceID,
}

#[derive(Debug)]
pub struct CameraComponent {
    fov: f32,
}
