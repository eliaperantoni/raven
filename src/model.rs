use std::cell::RefCell;
use std::error::Error;
use std::mem;

use russimp::mesh::Mesh;
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};

pub struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn from_file(path: &str) -> Result<Model, Box<dyn Error>> {
        let mut meshes = Vec::new();

        let mut scene = Scene::from_file(path, vec![
            PostProcess::Triangulate,
            PostProcess::FlipUVs,
        ])?;

        // TODO Error out on incomplete scene

        let root = match &scene.root {
            Some(root) => RefCell::borrow(root),
            None => return Err(Box::from(
                format!("model at {} has no root", path)
            ))
        };

        process_node(&root, &mut scene.meshes, &mut meshes);

        Ok(Model {
            meshes,
        })
    }
}

fn process_node(node: &Node, scene_meshes: &mut Vec<Mesh>, meshes: &mut Vec<Mesh>) {
    for mesh_idx in &node.meshes {
        let mut mesh = Mesh::default();
        mem::swap(&mut scene_meshes[*mesh_idx as usize], &mut mesh);

        meshes.push(mesh);
    }

    for child in &node.children {
        let child = RefCell::borrow(child);
        process_node(&child, scene_meshes, meshes);
    }
}
