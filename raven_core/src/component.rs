use std::any::{Any, TypeId};

use crate::ID;
use crate::material::Material;
use crate::mesh::Mesh;

#[derive(Debug)]
pub struct MeshComponent {
    pub entity: ID,
    // TODO Should probably use references instead of owned types
    pub mesh: Mesh,
    pub material: Material,
}

#[derive(Debug)]
pub struct CameraComponent {
    pub entity: ID,
    pub fov: f32,
}

pub trait Component {}
