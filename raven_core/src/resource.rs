use std::path::PathBuf;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Texture {
    pub raw: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub diffuse_tex: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
}
