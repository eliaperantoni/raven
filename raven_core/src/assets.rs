use std::path::PathBuf;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Texture {
    raw: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Material {
    diffuse_tex: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}
