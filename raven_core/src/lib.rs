#![feature(with_options)]

use std::error::Error;
use std::path::{Path, PathBuf};

use gl;
pub use glam;
pub use mat4;
use glam::{Mat4, Quat, Vec3};
use mat4::decompose;

use ecs::*;

use crate::component::{CameraComponent, HierarchyComponent, MeshComponent, TransformComponent};
use crate::io::Serializable;
use crate::resource::{Material, Mesh, Scene};
use crate::shader::Shader;
use crate::standard_shader::get_standard_shader;
use crate::vao::Vao;

pub mod ecs {
    pub use raven_ecs::*;
}

pub mod resource;
pub mod component;
pub mod io;
pub mod path;

mod vao;
mod shader;
mod standard_shader;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub struct Processor {
    project_root: PathBuf,

    scene: Option<Scene>,

    canvas_size: [u32; 2],
    shader: Shader,
}

impl Processor {
    pub fn new<R: AsRef<Path>>(project_root: R) -> Result<Processor> {
        Ok(Processor {
            project_root: project_root.as_ref().to_owned(),

            scene: None,

            canvas_size: [800, 400],
            shader: get_standard_shader()?,
        })
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, scene_path: P) -> Result<()> {
        let scene_path = path::as_fs_abs(&self.project_root, scene_path);

        self.scene = Some(Scene::load(scene_path)?);
        Ok(())
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas_size = [width, height];
    }

    fn clear_canvas(&self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn must_scene(&self) -> &Scene {
        self.scene.as_ref().expect("no scene loaded")
    }

    fn must_scene_mut(&mut self) -> &mut Scene {
        self.scene.as_mut().expect("no scene loaded")
    }

    pub fn do_frame(&mut self) -> Result<()> {
        self.clear_canvas();

        {
            let [width, height] = self.canvas_size;
            unsafe {
                gl::Viewport(0, 0, width as _, height as _);
            }
        }

        if self.scene.is_none() {
            return Ok(());
        }

        // First of all, we need to initialize a VAO for each MeshComponent that we haven't seen yet
        {
            let project_root = self.project_root.clone();

            for (_, (mut mesh_comp, ), _)
            in <(MeshComponent, )>::query_deep_mut(self.must_scene_mut()) {
                if mesh_comp.vao.is_some() { continue; };

                let mesh = Mesh::load(path::as_fs_abs(&project_root, &mesh_comp.mesh))?;
                let mat = Material::load(path::as_fs_abs(&project_root, &mesh_comp.mat))?;

                let vao = Vao::from(&mesh, &mat)?;

                mesh_comp.vao = Some(vao);
            }
        }

        let CameraMats { view_mat, projection_mat } = if let Some(mats) = self.compute_camera_mats() {
            mats
        } else {
            return Err(Box::from("no camera"));
        };

        // Now we can properly render them
        for (entity, (mesh_comp, ), _)
        in <(MeshComponent, )>::query_deep(self.must_scene()) {
            self.clear_canvas();

            let vao = mesh_comp.vao.as_ref().unwrap();

            self.shader.enable();
            self.shader.set_mat4("model", &self.combined_transform(entity));

            self.shader.set_mat4("view", &view_mat);
            self.shader.set_mat4("projection", &projection_mat);

            vao.draw();
        }

        Ok(())
    }

    fn combined_transform(&self, mut entity: Entity) -> Mat4 {
        let mut transform_components = Vec::new();

        loop {
            let transform_component = self.must_scene().get_one::<TransformComponent>(entity)
                .expect("entity does not have a transform component");
            transform_components.push(transform_component);

            let hierarchy_component = self.must_scene().get_one::<HierarchyComponent>(entity)
                .expect("entity does not have a hierarchy component");

            if let Some(parent_entity) = hierarchy_component.parent {
                entity = parent_entity;
            } else {
                break;
            }
        }

        let mut out = Mat4::IDENTITY;

        for transform_component in transform_components.into_iter() {
            out = out * transform_component.0;
        }

        out
    }
}

struct CameraMats {
    view_mat: Mat4,
    projection_mat: Mat4,
}

impl Processor {
    fn compute_camera_mats(&self) -> Option<CameraMats> {
        <(CameraComponent, )>::query_shallow(self.must_scene()).next().map(|(entity, _, _)| {
            let transform = self.combined_transform(entity);

            CameraMats {
                view_mat: {
                    let mut position = Vec3::default();
                    let mut scale = Vec3::default();
                    let mut rotation = Quat::default();

                    decompose(transform.as_ref(), position.as_mut(), scale.as_mut(), rotation.as_mut());

                    let forward = rotation.mul_vec3(-Vec3::Z).normalize();
                    let target = position + forward;

                    let right = Vec3::cross(forward, Vec3::Y).normalize();
                    let up = Vec3::cross(right, forward).normalize();

                    Mat4::look_at_rh(position, target, up)
                },
                projection_mat: {
                    let [width, height] = self.canvas_size;
                    let aspect_ratio = width as f32 / height as f32;
                    Mat4::perspective_rh_gl(90_f32.to_radians(), aspect_ratio, 0.1, 100.0)
                },
            }
        })
    }
}
