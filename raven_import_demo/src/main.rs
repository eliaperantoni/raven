#![feature(try_blocks)]

use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use image::GenericImageView;
use itertools::izip;
use md5::{Digest, Md5};
use russimp::material::PropertyTypeInfo;

use raven_core::component::{HierarchyComponent, MeshComponent, TransformComponent};
use raven_core::ecs::Entity;
use raven_core::glam::{Mat4, Vec2, Vec3, Vec4};
use raven_core::io::Serializable;
use raven_core::path;
use raven_core::resource::*;

const SCALE_FACTOR: Option<f32> = Some(0.01);

const PROJECT_ROOT_DIR: &'static str = "/home/elia/code/raven_proj";
const IMPORT_DIR: &'static str = ".import";

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
    if !path::is_valid(path.as_ref()) {
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

/// Given the absolute path to an asset, returns the path to the root directory for the imported files.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `$/.import/ferris/ferris.fbx`
fn as_import_root<P: AsRef<Path>>(path: P) -> PathBuf {
    assert!(path::is_valid(path.as_ref()));

    let mut import_root = PathBuf::default();
    import_root.push(path::PROJECT_ROOT_RUNE);
    import_root.push(IMPORT_DIR);
    import_root.push(path::strip_rune(path.as_ref()));

    import_root
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
    assert!(path::is_valid(path.as_ref()));

    let import_root = as_import_root(path);

    fs::create_dir_all(path::as_fs_abs(PROJECT_ROOT_DIR, &import_root)).map_err(|e| Box::<dyn Error>::from(e))?;

    // Make sure the import directory contains no file
    wipe_dir(path::as_fs_abs(PROJECT_ROOT_DIR, &import_root))?;

    Ok(import_root)
}

fn import_tex<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let import_root = prepare_import_root_for(path.as_ref())?;

    let tex = image::open(path::as_fs_abs(PROJECT_ROOT_DIR, path.as_ref()))?;

    let size = [tex.width(), tex.height()];

    let tex = tex.into_rgba8();

    let tex = Texture::new(tex.into_raw(), size);

    tex.save(path::as_fs_abs(PROJECT_ROOT_DIR, import_root.join("main.tex")))?;

    Ok(PathBuf::from(import_root.join("main.tex")))
}

struct SceneImporter<'a> {
    original_path: &'a Path,
    import_root: &'a Path,
    scene: &'a assimp::Scene,
    importing_scene: Scene,
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

        let fs_abs_path = path::as_fs_abs(PROJECT_ROOT_DIR, path.as_ref());
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
            import_root: &import_root,
            scene: &scene,
            importing_scene: Default::default(),
        };

        let root = scene
            .root
            .as_ref()
            .ok_or_else(|| Box::<dyn Error>::from("no root node"))?;
        let root = &*RefCell::borrow(Rc::borrow(root));

        let root_entity = importer.process_node(root, NodeTraversal::start(&root.name))?;

        if let Some(scale_factor) = SCALE_FACTOR {
            let w = &mut importer.importing_scene;

            let mut transform = w.get_one_mut::<TransformComponent>(root_entity).unwrap();
            let mat: &mut Mat4 = &mut transform.0;

            *mat = Mat4::from_scale(Vec3::ONE * scale_factor) * *mat;
        }

        importer.importing_scene.save(path::as_fs_abs(PROJECT_ROOT_DIR, import_root.join("main.scn")))?;

        Ok(())
    }

    fn process_node(&mut self, node: &assimp::Node, traversal: NodeTraversal) -> Result<Entity> {
        let entity = self.importing_scene.create();

        self.importing_scene.attach(entity, TransformComponent({
            let t = node.transformation;
            Mat4::from_cols(
                Vec4::new(t.a1, t.b1, t.c1, t.d1),
                Vec4::new(t.a2, t.b2, t.c2, t.d2),
                Vec4::new(t.a3, t.b3, t.c3, t.d3),
                Vec4::new(t.a4, t.b4, t.c4, t.d4),
            )
        }));
        self.importing_scene.attach(entity, HierarchyComponent::default());

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];

            let mesh_path = {
                let imported_mesh = self.extract_mesh(mesh)?;

                let mut hasher = Md5::default();
                Digest::update(&mut hasher, traversal.as_bytes());
                Digest::update(&mut hasher, &mesh.name);

                let mesh_file = format!("{:x}.mesh", hasher.finalize());
                imported_mesh.save(path::as_fs_abs(PROJECT_ROOT_DIR, self.import_root.join(&mesh_file)))?;

                self.import_root.join(&mesh_file)
            };

            let mat_path = {
                let mat = &self.scene.materials[mesh.material_index as usize];

                let mat_name = mat
                    .properties
                    .iter()
                    .find(|prop| prop.key == "")
                    .map(|prop| match &prop.data {
                        PropertyTypeInfo::String(s) => s,
                        _ => panic!("I expected the name of the material to be a string"),
                    });

                let mut imported_mat = Material { tex: None };

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

                        imported_mat.tex = Some(import_tex(tex_path)?);
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
                imported_mat.save(path::as_fs_abs(PROJECT_ROOT_DIR, self.import_root.join(&mat_file)))?;

                self.import_root.join(&mat_file)
            };

            self.importing_scene.attach(entity, MeshComponent::new(mesh_path, mat_path));
        }

        // Collects entities of children that we will later insert into the HierarchyComponent for this node
        let mut children_entities = Vec::new();

        for child in &node.children {
            let child = &*RefCell::borrow(Rc::borrow(child));
            let child_entity = self.process_node(child, traversal.descend(&child.name))?;

            let mut hierarchy_component = self.importing_scene.get_one_mut::<HierarchyComponent>(child_entity).unwrap();
            hierarchy_component.parent = Some(entity);

            children_entities.push(child_entity);
        }

        let mut hierarchy_component = self.importing_scene.get_one_mut::<HierarchyComponent>(entity).unwrap();
        hierarchy_component.children = children_entities;

        Ok(entity)
    }

    fn extract_mesh(&self, mesh: &assimp::Mesh) -> Result<Mesh> {
        let iter = izip!(
            mesh.vertices.iter(),
            mesh.normals.iter(),
            mesh.texture_coords[0].as_ref().expect("missing 0-th uv channel").iter(),
        );

        let vertices: Vec<_> = iter
            .map(|(position, normal, uv)| Vertex {
                position: Vec3::new(position.x, position.y, position.z),
                normal: Vec3::new(normal.x, normal.y, normal.z),
                uv: Vec2::new(uv.x, uv.y),
            })
            .collect();

        let indices: Vec<_> = mesh
            .faces
            .iter()
            .map(|face| {
                // Should be true, we told Assimp to triangulate
                assert_eq!(face.0.len(), 3);
                face.0.iter().map(|idx| *idx)
            })
            .flatten()
            .collect();

        Ok(Mesh { vertices, indices })
    }
}
