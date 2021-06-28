use crate::material::Material;
use crate::mesh::Mesh;

#[derive(Debug)]
pub struct MeshComponent {
    // TODO Should probably use references instead of owned types
    mesh: Mesh,
    material: Material,
}

#[derive(Debug)]
pub struct CameraComponent {
    fov: f64,
}

#[derive(Debug)]
pub enum Component {
    Mesh (MeshComponent),
    Camera (CameraComponent),
}
