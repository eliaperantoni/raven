use std::any::Any;

use crate::material::Material;
use crate::mesh::Mesh;

#[derive(Debug)]
pub struct MeshComponent {
    // TODO Should probably use references instead of owned types
    pub mesh: Mesh,
    pub material: Material,
}

#[derive(Debug)]
pub struct CameraComponent {
    pub fov: f64,
}

impl Default for CameraComponent {
    fn default() -> Self {
        CameraComponent {
            fov: 90.0,
        }
    }
}

pub enum Component {
    Mesh(MeshComponent),
    Camera(CameraComponent),
}

impl Component {
    pub fn as_any(&self) -> &dyn Any {
        use Component::*;
        match self {
            Mesh(val) => val,
            Camera(val) => val,
        }
    }

    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        use Component::*;
        match self {
            Mesh(val) => val,
            Camera(val) => val,
        }
    }
}

impl From<MeshComponent> for Component {
    fn from(val: MeshComponent) -> Self {
        Component::Mesh(val)
    }
}

impl From<CameraComponent> for Component {
    fn from(val: CameraComponent) -> Self {
        Component::Camera(val)
    }
}
