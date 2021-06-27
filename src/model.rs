use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::iter;
use std::mem;
use std::path::PathBuf;
use std::ptr;

use glam::{Vec2, Vec3};
use itertools::izip;

use crate::material::{self, Material};
use crate::mesh::{self, Mesh, Vertex};
use crate::entity::Entity;
use crate::component::Component;

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

        Ok(loader.process_node(&root))
    }

    fn process_node(&self, node: &russimp::Node) -> Entity {
        let mut entity = Entity::default();

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];
            let mesh = mesh::from_assimp(mesh);

            entity.add_component(Component::MeshComponent(mesh));
        }

        for child in &node.children {
            let child = RefCell::borrow(child);
            // TODO Apply node transform to child entity
            let child = self.process_node(&child);

            entity.add_child(child);
        }

        println!("ciao");
        entity
    }
}
