use gltf;
use glam::{Vec2, Vec3};
use std::error::Error;
use itertools::izip;

pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    tex_coords: Vec2,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub struct Model {
    meshes: Vec<Mesh>,
}

pub struct ModelLoader {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

impl ModelLoader {
    pub fn load_file(path: &str) -> Result<Model, Box<dyn Error>> {
        let (document, buffers, images) = gltf::import(path)?;

        let model_loader = ModelLoader {
            document,
            buffers,
            images,
        };

        let default_scene = match document.default_scene() {
            Some(scene) => scene,
            None => return Err(Box::from("no default scene"))
        };

        let mut meshes = Vec::new();

        for node in default_scene.nodes() {
            meshes.extend(model_loader.process_node(node)?);
        }

        Ok(Model {
            meshes,
        })
    }

    fn process_node(&self, node: gltf::Node) -> Result<Vec<Mesh>, Box<dyn Error>> {
        let mut meshes = Vec::new();

        if let Some(mesh) = node.mesh() {
            meshes.push(self.read_mesh(mesh)?);
        }

        for node in node.children() {
            meshes.extend(self.process_node(node)?);
        }

        Ok(meshes)
    }

    fn read_mesh(&self, mesh: gltf::Mesh) -> Result<Mesh, Box<dyn Error>> {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(
                |buffer| Some(&self.buffers[buffer.index()])
            );

            let positions_iter = reader.read_positions().ok_or(
                Box::from("no positions in primitive")
            )?;

            let normals_iter = reader.read_normals().ok_or(
                Box::from("no normals in primitive")
            )?;

            let tex_coords_iter = reader.read_tex_coords(0).ok_or(
                Box::from("no text coords in primitive")
            )?;

            for (position, normal, tex_coords) in izip!(
                positions_iter,
                normals_iter,
                tex_coords_iter,
            ) {
                let x: () = position;
            }
        }

        todo!()
    }
}
