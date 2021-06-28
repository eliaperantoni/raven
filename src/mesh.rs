use std::error::Error;
use std::iter;

use glam::{Vec2, Vec3};
use itertools::izip;

mod russimp {
    pub use russimp::mesh::Mesh;
}

#[derive(Debug)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

#[derive(Debug)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

pub fn from_assimp(mesh: &russimp::Mesh) -> Result<Mesh, Box<dyn Error>> {
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

    Ok(Mesh {
        vertices,
        indices,
    })
}
