use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use image::GenericImageView;
use itertools::izip;
use md5::{Digest, Md5};

use raven_core::component::{HierarchyComponent, MeshComponent, TransformComponent};
use raven_core::ecs::Entity;
use raven_core::glam::{Mat4, Vec2, Vec3, Vec4};
use raven_core::io::Serializable;
use raven_core::path as path_pkg;
use raven_core::resource::{Material, Mesh, Scene, Texture, Vertex};

use crate::OpenProjectState;
use crate::Result;

mod assimp {
    pub use russimp::material::Material;
    pub use russimp::material::PropertyTypeInfo;
    pub use russimp::mesh::Mesh;
    pub use russimp::node::Node;
    pub use russimp::scene::{PostProcess, Scene};
    pub use russimp::texture::TextureType;
}

const SCALE_FACTOR: Option<f32> = Some(0.01);

const IMPORT_DIR: &'static str = ".import";

pub(super) fn import(path: &Path, state: &OpenProjectState) -> Result<()> {
    if !path_pkg::is_valid(path) {
        panic!("invalid path: {:?}", path);
    }

    let ext = match path.extension() {
        Some(os_ext) => os_ext.to_str(),
        _ => return Err(Box::<dyn Error>::from("no extension")),
    };

    match ext {
        Some("png" | "jpg" | "jpeg") => import_tex(path, state),
        Some("fbx" | "obj") => SceneImporter::import(path, state),
        _ => return Err(Box::<dyn Error>::from("unknown extension")),
    }?;

    Ok(())
}

/// Given the absolute path to an asset, returns the path to the root directory for the imported files.
///
/// For instance:
/// `$/ferris/ferris.fbx` becomes `$/.import/ferris/ferris.fbx`
fn as_import_root(path: &Path) -> PathBuf {
    assert!(path_pkg::is_valid(path));

    let mut import_root = PathBuf::default();
    import_root.push(path_pkg::PROJECT_ROOT_RUNE);
    import_root.push(IMPORT_DIR);
    import_root.push(path_pkg::strip_rune(path));

    import_root
}

fn wipe_dir(path: &Path) -> Result<()> {
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

fn prepare_import_root_for(path: &Path, state: &OpenProjectState) -> Result<PathBuf> {
    assert!(path_pkg::is_valid(path));

    let import_root = as_import_root(path);

    fs::create_dir_all(path_pkg::as_fs_abs(&state.project_root, &import_root))
        .map_err(|e| Box::<dyn Error>::from(e))?;

    // Make sure the import directory contains no file
    wipe_dir(&path_pkg::as_fs_abs(&state.project_root, &import_root))?;

    Ok(import_root)
}

fn import_tex(path: &Path, state: &OpenProjectState) -> Result<()> {
    let import_root = prepare_import_root_for(path, state)?;

    let tex = image::open(path_pkg::as_fs_abs(&state.project_root, path))?;

    let size = [tex.width(), tex.height()];

    let tex = tex.into_rgba8();

    let tex = Texture::new(tex.into_raw(), size);

    tex.save(path_pkg::as_fs_abs(
        &state.project_root,
        import_root.join("main.tex"),
    ))?;

    Ok(())
}

struct SceneImporter<'me> {
    import_root: &'me Path,
    scene: &'me assimp::Scene,
    importing_scene: Scene,
}

struct NodeTraversal(Vec<String>);

impl NodeTraversal {
    fn start(root: &str) -> NodeTraversal {
        NodeTraversal(vec![root.to_owned()])
    }

    fn descend(&self, node: &str) -> NodeTraversal {
        let mut vec = self.0.clone();
        vec.push(node.to_owned());
        NodeTraversal(vec)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.0.join("/").into_bytes()
    }
}

impl<'me> SceneImporter<'me> {
    fn import(path: &Path, state: &OpenProjectState) -> Result<()> {
        let import_root = prepare_import_root_for(path, state)?;

        let fs_abs_path = path_pkg::as_fs_abs(&state.project_root, path);
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
            import_root: &import_root,
            scene: &scene,
            importing_scene: Default::default(),
        };

        let root = scene
            .root
            .as_ref()
            .ok_or_else(|| Box::<dyn Error>::from("no root node"))?;
        let root = &*RefCell::borrow(Rc::borrow(root));

        let root_entity = importer.process_node(root, NodeTraversal::start(&root.name), state)?;

        if let Some(scale_factor) = SCALE_FACTOR {
            let w = &mut importer.importing_scene;

            let mut transform = w.get_one_mut::<TransformComponent>(root_entity).unwrap();
            let mat: &mut Mat4 = &mut transform.0;

            *mat = Mat4::from_scale(Vec3::ONE * scale_factor) * *mat;
        }

        importer.importing_scene.save(path_pkg::as_fs_abs(
            &state.project_root,
            import_root.join("main.scn"),
        ))?;

        Ok(())
    }

    fn process_node(
        &mut self,
        node: &assimp::Node,
        traversal: NodeTraversal,
        state: &OpenProjectState,
    ) -> Result<Entity> {
        let entity = self.importing_scene.create();

        self.importing_scene.attach(
            entity,
            TransformComponent({
                let t = node.transformation;
                Mat4::from_cols(
                    Vec4::new(t.a1, t.b1, t.c1, t.d1),
                    Vec4::new(t.a2, t.b2, t.c2, t.d2),
                    Vec4::new(t.a3, t.b3, t.c3, t.d3),
                    Vec4::new(t.a4, t.b4, t.c4, t.d4),
                )
            }),
        );
        self.importing_scene
            .attach(entity, HierarchyComponent::default());

        for mesh_idx in &node.meshes {
            let mesh = &self.scene.meshes[*mesh_idx as usize];

            let mesh_path = {
                let imported_mesh = self.extract_mesh(mesh)?;

                let mut hasher = Md5::default();
                Digest::update(&mut hasher, traversal.as_bytes());
                Digest::update(&mut hasher, &mesh.name);

                let mesh_file = format!("{:x}.mesh", hasher.finalize());
                imported_mesh.save(path_pkg::as_fs_abs(
                    &state.project_root,
                    self.import_root.join(&mesh_file),
                ))?;

                self.import_root.join(&mesh_file)
            };

            let mat_path = {
                let mat = &self.scene.materials[mesh.material_index as usize];

                let mat_name = mat
                    .properties
                    .iter()
                    .find(|prop| prop.key == "?mat.name")
                    .map(|prop| match &prop.data {
                        assimp::PropertyTypeInfo::String(s) => s,
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
                        let fs_path = PathBuf::from(&tex.path);
                        if !fs_path.is_relative() {
                            return Err(Box::<dyn Error>::from(
                                "textures paths must be relative to the scene file",
                            ));
                        }

                        // Let's say the we are importing `cube.fbx` from this filesystem:
                        // models
                        //  |_ cube/
                        //      |_ cube.fbx
                        //      |_ textures/
                        //          |_ diffuse.png
                        // and that `cube.fbx` refers to its texture using a relative path of `./texture/diffuse.png`.
                        // Then, the import path of `$/models/cube/cube.fbx` will be `$/.import/models/cube/cube.fbx`
                        // and the import path of `$/models/cube/textures/diffuse.png` will be
                        // `$/.import/models/cube/textures/diffuse.png`. As you can see to obtain it we can simply
                        // pop from the import root of the scene and append the relative texture path. That will match
                        // the import root used when importing the texture

                        let mut raven_path = self.import_root.to_owned();
                        raven_path.pop();
                        raven_path.push(&fs_path);
                        raven_path.push("main.tex");

                        imported_mat.tex = Some(raven_path);
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
                imported_mat.save(path_pkg::as_fs_abs(
                    &state.project_root,
                    self.import_root.join(&mat_file),
                ))?;

                self.import_root.join(&mat_file)
            };

            self.importing_scene
                .attach(entity, MeshComponent::new(mesh_path, mat_path));
        }

        // Collects entities of children that we will later insert into the HierarchyComponent for this node
        let mut children_entities = Vec::new();

        for child in &node.children {
            let child = &*RefCell::borrow(Rc::borrow(child));
            let child_entity = self.process_node(child, traversal.descend(&child.name), state)?;

            let mut hierarchy_component = self
                .importing_scene
                .get_one_mut::<HierarchyComponent>(child_entity)
                .unwrap();
            hierarchy_component.parent = Some(entity);

            children_entities.push(child_entity);
        }

        let mut hierarchy_component = self
            .importing_scene
            .get_one_mut::<HierarchyComponent>(entity)
            .unwrap();
        hierarchy_component.children = children_entities;

        Ok(entity)
    }

    fn extract_mesh(&self, mesh: &assimp::Mesh) -> Result<Mesh> {
        let iter = izip!(
            mesh.vertices.iter(),
            mesh.normals.iter(),
            mesh.texture_coords[0]
                .as_ref()
                .expect("missing 0-th uv channel")
                .iter(),
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
