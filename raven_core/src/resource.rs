use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use raven_ecs::World;

#[derive(Serialize, Deserialize)]
pub struct Texture {
    pub raw: Vec<u8>,
    pub size: [u32; 2],

    #[serde(skip)]
    pub(crate) id: Option<u32>,
}

impl Texture {
    pub fn new(raw: Vec<u8>, size: [u32; 2]) -> Texture {
        Texture {
            raw,
            size,
            id: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub tex: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Scene(World);

impl Deref for Scene {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
