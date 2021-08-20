#![feature(try_blocks)]

use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::iter;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use itertools::izip;

use raven_core::glam::{Vec2, Vec3};
use raven_core::io::Serializable;
use raven_core::resource::*;

const PROJECT_ROOT: &'static str = "/home/elia/code/raven_proj";
const IMPORT_DIR: &'static str = ".import";

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

mod assimp {
    pub use russimp::material::Material;
    pub use russimp::mesh::Mesh;
    pub use russimp::node::Node;
    pub use russimp::scene::{PostProcess, Scene};
}

fn main() -> Result<()> {
    import("ferris/ferris.fbx")?;
    Ok(())
}

fn import<P: AsRef<Path>>(path: P) -> Result<()> {
    let ext = match path.as_ref().extension() {
        Some(os_ext) => os_ext.to_str(),
        _ => return Err(Box::<dyn Error>::from("no extension")),
    };

    match ext {
        Some("png" | "jpg" | "jpeg") => import_tex(path.as_ref()),
        Some("fbx" | "obj") => SceneImporter::import(path.as_ref()),
        _ => return Err(Box::<dyn Error>::from("unknown extension")),
    }?;

    Ok(())
}

fn as_import_root<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut import_root = PathBuf::default();
    import_root.push(PROJECT_ROOT);
    import_root.push(IMPORT_DIR);
    import_root.push(path.as_ref());

    import_root
}

fn as_abs<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut abs_path = PathBuf::default();
    abs_path.push(PROJECT_ROOT);
    abs_path.push(path.as_ref());

    abs_path
}

fn wipe_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            wipe_dir(&path)?;
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn prepare_import_root<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let import_root = as_import_root(path);
    fs::create_dir_all(&import_root).map_err(|e| Box::<dyn Error>::from(e))?;

    // Make sure the import directory contains no file
    wipe_dir(&import_root)?;

    Ok(import_root)
}

fn import_tex<P: AsRef<Path>>(path: P) -> Result<()> {
    let import_root = prepare_import_root(path.as_ref())?;

    let abs_path = as_abs(path.as_ref());

    let tex = image::open(&abs_path)?;
    let tex = tex.into_rgba8();

    let tex = Texture {
        raw: tex.into_raw(),
    };

    let dst_path = import_root.join("main.tex");
    tex.save(dst_path)?;

    Ok(())
}

struct SceneImporter<'a> {
    scene: &'a assimp::Scene,
    import_root: PathBuf,
}

impl<'a> SceneImporter<'a> {
    fn import<P: AsRef<Path>>(path: P) -> Result<()> {
        let import_root = prepare_import_root(path.as_ref())?;

        let abs_path = as_abs(path.as_ref());
        let abs_path = abs_path
            .to_str()
            .ok_or_else(|| Box::<dyn Error>::from("assimp requires unicode path"))?;

        let scene = assimp::Scene::from_file(abs_path, vec![
            assimp::PostProcess::GenerateNormals,
            assimp::PostProcess::Triangulate,
        ])?;

        let importer = SceneImporter {
            scene: &scene,
            import_root
        };

        let root = scene
            .root
            .as_ref()
            .ok_or_else(|| Box::<dyn Error>::from("no root node"))?;
        let root = &*RefCell::borrow(Rc::borrow(root));

        importer.process_node(root)
    }

    fn process_node(&self, node: &assimp::Node) -> Result<()> {
        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];

            let mesh = self.extract_mesh(mesh)?;
            mesh.save(self.import_root.join("some.mesh"))?;
        }

        for child in &node.children {
            let child = &*RefCell::borrow(Rc::borrow(child));
            self.process_node(child)?;
        }

        Ok(())
    }

    fn extract_mesh(&self, mesh: &assimp::Mesh) -> Result<Mesh> {
        let uvs = try {
            let vec = mesh.texture_coords.get(0)?;
            let vec = vec.as_ref()?;
            vec
        };

        let iter = izip!(
            mesh.vertices.iter(),
            mesh.normals.iter(),
        );

        let vertices: Vec<_> = iter
            .enumerate()
            .map(|(i, (position, normal))| Vertex {
                position: Vec3::new(position.x, position.y, position.z),
                normal: Vec3::new(normal.x, normal.y, normal.z),
                uv: if let Some(uvs) = uvs {
                    let uv = uvs[i];
                    Some(Vec2::new(uv.x, uv.y))
                } else {
                    None
                },
            })
            .collect();

        let indices: Vec<_> = mesh
            .faces
            .iter()
            .map(|face| {
                // Should be true, we told Assimp to triangulate
                assert_eq!(face.0.len(), 3);
                face.0.iter().map(|idx| *idx as usize)
            })
            .flatten()
            .collect();

        Ok(Mesh { vertices, indices })
    }
}
