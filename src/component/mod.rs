use camera::Camera;
use mesh::Mesh;

pub mod camera;
pub mod mesh;

pub enum Component {
    Mesh(Mesh),
    Camera(Camera),
}
