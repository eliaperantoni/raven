use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::mem;

use glam::{Vec2, Vec3};

mod assimp {
    pub use assimp::import::Importer;
    pub use assimp::math::Vector3D;
    pub use assimp::scene::{Mesh, Node, Scene};
    pub use assimp_sys::aiGetMaterialTexture as get_material_texture;
}

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

impl Model {
    pub fn from_file(path: &str) -> Result<Model, Box<dyn Error>> {
        let mut meshes = Vec::new();

        let mut importer = assimp::Importer::new();
        importer.flip_uvs(true);
        importer.triangulate(true);

        let mut scene = importer.read_file(path)?;

        if scene.is_incomplete() {
            return Err(Box::from(
                format!("model at {} is incomplete", path)
            ));
        }

        process_node(&scene.root_node(), &scene, &mut meshes);

        Ok(Model {
            meshes,
        })
    }
}

fn process_node(node: &assimp::Node, scene: &assimp::Scene, meshes: &mut Vec<Mesh>) {
    for mesh_idx in node.meshes() {
        let mesh = &scene.mesh(*mesh_idx as _).expect(&format!("mesh with id {} not found in scene", mesh_idx));
        meshes.push(process_mesh(mesh, scene));
    }

    for child in node.child_iter() {
        process_node(&child, scene, meshes);
    }
}

fn process_mesh(mesh: &assimp::Mesh, scene: &assimp::Scene) -> Mesh {
    let vertices: Vec<_> = mesh
        .vertex_iter()
        .zip(mesh.normal_iter())
        .zip(mesh.texture_coords_iter(0))
        .map(|((position, normal), tex_coords)| {
            Vertex {
                position: Vec3::new(position.x, position.y, position.z),
                normal: Vec3::new(normal.x, normal.y, normal.z),
                tex_coords: Vec2::new(tex_coords.x, tex_coords.y),
            }
        })
        .collect();

    let indices: Vec<_> = mesh.face_iter().flat_map(|face| {
        // Should always be true because we told Assimp to triangulate
        assert_eq!(face.num_indices, 3);
        (0..face.num_indices).map(move |i| face[i as _])
    }).collect();

    dbg!(&scene.materials);

    if mesh.material_index > 0 {
        let mat = scene.material_iter().nth(mesh.material_index as _).unwrap();
        assimp::get_material_texture(mat, )
    }

    Mesh {
        vertices,
        indices,
    }
}
