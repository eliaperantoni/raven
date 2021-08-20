#![feature(try_blocks)]

use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use itertools::izip;
use md5::{Digest, Md5};
use russimp::material::PropertyTypeInfo;

use raven_core::glam::{Vec2, Vec3};
use raven_core::io::Serializable;
use raven_core::resource::*;
use raven_ecs::*;

const PROJECT_ROOT_RUNE: &'static str = "$/";
const IMPORT_DIR: &'static str = ".import";

const PROJECT_ROOT: &'static str = "/home/elia/code/raven_proj";

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

mod assimp {
    pub use russimp::material::Material;
    pub use russimp::mesh::Mesh;
    pub use russimp::node::Node;
    pub use russimp::scene::{PostProcess, Scene};
    pub use russimp::texture::TextureType;
}

fn main() -> Result<()> {
    import("$/ferris/ferris.fbx")?;
    Ok(())
}

fn import<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().starts_with(PROJECT_ROOT_RUNE) {
        panic!("support is for absolute paths only");
    }

    let ext = match path.as_ref().extension() {
        Some(os_ext) => os_ext.to_str(),
        _ => return Err(Box::<dyn Error>::from("no extension")),
    };

    match ext {
        Some("png" | "jpg" | "jpeg") => import_tex(path.as_ref()).map(|_| ()),
        Some("fbx" | "obj") => SceneImporter::import(path.as_ref()),
        _ => return Err(Box::<dyn Error>::from("unknown extension")),
    }?;

    Ok(())
}

fn strip_rune<P: AsRef<Path> + ?Sized>(path: &P) -> &Path {
    path.as_ref()
        .strip_prefix(PROJECT_ROOT_RUNE)
        .expect("expected to find project root rune to strip it")
}

/// Given the absolute path to an asset, returns the path to the root directory for the imported files.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `$/.import/ferris/ferris.fbx`
fn as_import_root<P: AsRef<Path>>(path: P) -> PathBuf {
    assert!(path.as_ref().starts_with(PROJECT_ROOT_RUNE));

    let mut import_root = PathBuf::default();
    import_root.push(PROJECT_ROOT_RUNE);
    import_root.push(IMPORT_DIR);
    import_root.push(strip_rune(path.as_ref()));

    import_root
}

/// Given the absolute path to an asset, returns the filesystem absolute path.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `$(pwd)/$PROJECT_DIR/ferris/ferris.fbx`
fn as_fs_abs<P: AsRef<Path>>(path: P) -> PathBuf {
    assert!(path.as_ref().starts_with(PROJECT_ROOT_RUNE));

    let mut abs_path = PathBuf::default();
    abs_path.push(PROJECT_ROOT);
    abs_path.push(strip_rune(path.as_ref()));

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

fn prepare_import_root_for<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    assert!(path.as_ref().starts_with(PROJECT_ROOT_RUNE));

    let import_root = as_import_root(path);

    fs::create_dir_all(as_fs_abs(&import_root)).map_err(|e| Box::<dyn Error>::from(e))?;

    // Make sure the import directory contains no file
    wipe_dir(as_fs_abs(&import_root))?;

    Ok(import_root)
}

fn import_tex<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let import_root = prepare_import_root_for(path.as_ref())?;

    let tex = image::open(as_fs_abs(path.as_ref()))?;
    let tex = tex.into_rgba8();

    let tex = Texture {
        raw: tex.into_raw(),
    };

    tex.save(as_fs_abs(import_root.join("main.tex")))?;

    Ok(PathBuf::from(import_root.join("main.tex")))
}

struct SceneImporter<'a> {
    original_path: &'a Path,
    import_root: &'a Path,
    scene: &'a assimp::Scene,
    world: World,
}

struct NodeTraversal(Vec<String>);

impl NodeTraversal {
    fn start<S: AsRef<str>>(root: S) -> NodeTraversal {
        NodeTraversal(vec![root.as_ref().to_owned()])
    }

    fn descend<S: AsRef<str>>(&self, node: S) -> NodeTraversal {
        let mut vec = self.0.clone();
        vec.push(node.as_ref().to_owned());
        NodeTraversal(vec)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.0.join("/").into_bytes()
    }
}

impl<'a> SceneImporter<'a> {
    fn import<P: AsRef<Path>>(path: P) -> Result<()> {
        let import_root = prepare_import_root_for(path.as_ref())?;

        let fs_abs_path = as_fs_abs(path.as_ref());
        let fs_abs_path = fs_abs_path
            .to_str()
            .ok_or_else(|| Box::<dyn Error>::from("assimp requires unicode path"))?;

        let scene = assimp::Scene::from_file(
            fs_abs_path,
            vec![
                assimp::PostProcess::GenerateNormals,
                assimp::PostProcess::Triangulate,
            ],
        )?;

        let mut importer = SceneImporter {
            original_path: path.as_ref(),
            scene: &scene,
            import_root: &import_root,
            world: World::default(),
        };

        let root = scene
            .root
            .as_ref()
            .ok_or_else(|| Box::<dyn Error>::from("no root node"))?;
        let root = &*RefCell::borrow(Rc::borrow(root));

        importer.process_node(root, NodeTraversal::start(&root.name))
    }

    fn process_node(&mut self, node: &assimp::Node, traversal: NodeTraversal) -> Result<()> {
        let entity = self.world.create();

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];

            {
                let imported_mesh = self.extract_mesh(mesh)?;

                let mut hasher = Md5::default();
                Digest::update(&mut hasher, traversal.as_bytes());
                Digest::update(&mut hasher, &mesh.name);

                let mesh_file = format!("{:x}.mesh", hasher.finalize());
                imported_mesh.save(as_fs_abs(self.import_root.join(mesh_file)))?;
            }

            let mat = &self.scene.materials[mesh.material_index as usize];

            {
                let mat_name = mat
                    .properties
                    .iter()
                    .find(|prop| prop.key == "")
                    .map(|prop| match &prop.data {
                        PropertyTypeInfo::String(s) => s,
                        _ => panic!("I expected the name of the material to be a string"),
                    });

                let mut imported_mat = Material { diffuse_tex: None };

                match mat
                    .textures
                    .get(&assimp::TextureType::Diffuse)
                    .map(|tex_vec| tex_vec.first())
                    .flatten()
                {
                    Some(tex) => {
                        let tex_path = {
                            let mut scene_wd = PathBuf::from(self.original_path);
                            scene_wd.pop();
                            scene_wd.push(&tex.path);
                            scene_wd
                        };

                        imported_mat.diffuse_tex = Some(import_tex(tex_path)?);
                    }
                    _ => (),
                }

                let mut hasher = Md5::default();

                if let Some(mat_name) = mat_name {
                    // If the material has a name, use it as the hash
                    Digest::update(&mut hasher, mat_name.as_bytes());
                } else {
                    // Otherwise use the same string used for the mesh but append "/MATERIAL"
                    Digest::update(&mut hasher, traversal.as_bytes());
                    Digest::update(&mut hasher, &mesh.name);
                    Digest::update(&mut hasher, "/MATERIAL");
                }

                let mat_file = format!("{:x}.mat", hasher.finalize());
                imported_mat.save(as_fs_abs(self.import_root.join(mat_file)))?;
            }
        }

        for child in &node.children {
            let child = &*RefCell::borrow(Rc::borrow(child));
            self.process_node(child, traversal.descend(&child.name))?;
        }

        Ok(())
    }

    fn extract_mesh(&self, mesh: &assimp::Mesh) -> Result<Mesh> {
        let uvs = try {
            let vec = mesh.texture_coords.get(0)?;
            let vec = vec.as_ref()?;
            vec
        };

        let iter = izip!(mesh.vertices.iter(), mesh.normals.iter());

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
