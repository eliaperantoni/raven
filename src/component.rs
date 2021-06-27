use crate::material::Material;
use crate::mesh::Mesh;

pub enum Component {
    Mesh {
        // TODO Should probably use references instead of owned types
        mesh: Mesh,
        material: Material,
    },
    Camera {
        fov: f64,
    },
}
