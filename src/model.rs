use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::iter;
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;

use glam::{Vec2, Vec3};
use itertools::izip;

use crate::entity::Entity;
use crate::component::MeshComponent;
use crate::mesh::{self, Mesh, Vertex};
use crate::material::{self, Material};

mod russimp {
    pub use russimp::mesh::Mesh;
    pub use russimp::node::Node;
    pub use russimp::scene::{PostProcess, Scene};
    pub use russimp::texture::TextureType;
    pub use russimp::Vector3D;
}

pub struct ModelLoader {
    scene: russimp::Scene,
    base_dir: PathBuf,
}

impl ModelLoader {
    pub fn from_file(path: &str) -> Result<Entity, Box<dyn Error>> {
        let scene = russimp::Scene::from_file(path, vec![
            russimp::PostProcess::Triangulate,
            russimp::PostProcess::FlipUVs,
        ])?;

        let mut loader = ModelLoader {
            scene,
            base_dir: {
                let mut base_dir = PathBuf::from(path);
                base_dir.pop();
                base_dir
            },
        };

        let root = loader.scene.root.clone().ok_or(
            Box::<dyn Error>::from("no root node")
        )?;

        let root = RefCell::borrow(&root);

        loader.process_node(&root)
    }

    fn process_node(&self, node: &russimp::Node) -> Result<Entity, Box<dyn Error>> {
        let mut entity = Entity::default();

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];
            let material = &self.scene.materials[mesh.material_index as usize];

            let mesh = mesh::from_assimp(mesh)?;
            let material = material::from_assimp(material, &self.base_dir)?;

            entity.add_component(MeshComponent {
                mesh,
                material,
            }.into());
        }

        for child in &node.children {
            let child = RefCell::borrow(child);
            // TODO Apply node transform to child entity
            let child = self.process_node(&child)?;

            entity.add_child(child);
        }

        Ok(entity)
    }
}
