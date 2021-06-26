use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::iter;
use std::mem;
use std::ptr;
use std::path::PathBuf;

use glam::{Vec2, Vec3};
use itertools::izip;

use super::material::{self, Material};

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

pub struct Model {
    meshes: Vec<Mesh>,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

impl ModelLoader {
    pub fn from_file(path: &str) -> Result<Model, Box<dyn Error>> {
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

        let meshes = loader.process_node(&root);

        Ok(Model {
            meshes,
        })
    }

    fn process_node(&self, node: &russimp::Node) -> Vec<Mesh> {
        let mut meshes = Vec::new();

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];
            let mesh = self.process_mesh(mesh);

            meshes.push(mesh);
        }

        for child in &node.children {
            let child = RefCell::borrow(child);
            meshes.extend(self.process_node(&child));
        }

        meshes
    }

    fn process_mesh(&self, mesh: &russimp::Mesh) -> Mesh {
        // Iterator over the UV coordinates. If no UVs are present, it is an infinite iterator that keeps returning None
        let uvs_iter: Box<dyn Iterator<Item=_>> =
            match &mesh.texture_coords[0] {
                Some(uvs) => Box::new(uvs.iter().map(|uv| Some(uv))),
                None => Box::new(iter::repeat(None)),
            };

        let iter = izip!(
            mesh.vertices.iter(),
            mesh.normals.iter(),
            uvs_iter,
        );

        let vertices: Vec<_> = iter.map(|(position, normal, uv)| {
            Vertex {
                position: Vec3::new(position.x, position.y, position.z),
                normal: Vec3::new(normal.x, normal.y, normal.z),
                uv: if let Some(uv) = uv {
                    Vec2::new(uv.x, uv.y)
                } else {
                    Vec2::new(0.0, 0.0)
                },
            }
        }).collect();

        let indices: Vec<_> = mesh.faces.iter().map(|face| {
            // Should be true, we told Assimp to triangulate
            assert_eq!(face.0.len(), 3);
            face.0.iter().copied()
        }).flatten().collect();

        if mesh.material_index >= 0 {
            let mat = &self.scene.materials[mesh.material_index as usize];
            material::from_russimp(mat, &self.base_dir);
        }

        Mesh {
            vertices,
            indices,
        }
    }
}
