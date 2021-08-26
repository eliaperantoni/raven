#![feature(with_options)]

use std::error::Error;
use std::path::{Path, PathBuf};

use gl;
pub use glam;
use glam::{Mat4, Quat, Vec3};
pub use mat4;
use mat4::decompose;

use ecs::*;

use crate::component::{CameraComponent, HierarchyComponent, MeshComponent, SceneComponent, TransformComponent};
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
    state: ProcessorState,
    scene: Option<Scene>,
}

struct ProcessorState {
    project_root: PathBuf,
    canvas_size: [u32; 2],
    shader: Shader,
    camera_mats: Option<CameraMats>,
}

struct CameraMats {
    view_mat: Mat4,
    projection_mat: Mat4,
}

#[derive(Default)]
struct Context {
    transform: Mat4,
}

impl Processor {
    pub fn new<R: AsRef<Path>>(project_root: R) -> Result<Processor> {
        Ok(Processor {
            state: ProcessorState {
                project_root: project_root.as_ref().to_owned(),
                canvas_size: [800, 400],
                shader: get_standard_shader()?,
                camera_mats: None,
            },
            scene: None,
        })
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, scene_path: P) -> Result<()> {
        let scene_path = path::as_fs_abs(&self.state.project_root, scene_path);

        self.scene = Some(Scene::load(scene_path)?);
        Ok(())
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.state.canvas_size = [width, height];
    }

    pub fn do_frame(&mut self) -> Result<()> {
        clear_canvas();

        {
            let [width, height] = self.state.canvas_size;
            unsafe {
                gl::Viewport(0, 0, width as _, height as _);
            }
        }

        if self.scene.is_none() {
            return Ok(());
        }

        self.state.camera_mats = Some(self.compute_camera_mats().ok_or_else(|| Box::<dyn Error>::from("no camera"))?);

        Processor::process_scene(self.scene.as_mut().unwrap(), &mut self.state, Context::default())?;

        Ok(())
    }

    fn process_scene(scene: &mut Scene, state: &mut ProcessorState, ctx: Context) -> Result<()> {
        for (_, (mut scene_comp, transform_comp), _)
        in <(SceneComponent, TransformComponent)>::query_shallow_mut(scene) {
            if scene_comp.loaded.is_none() {
                scene_comp.loaded = Some(Scene::load(path::as_fs_abs(&state.project_root, &scene_comp.scene))?);
            }

            Processor::process_scene(scene_comp.loaded.as_mut().unwrap(), state, Context {
                transform: transform_comp.0 * ctx.transform,
            })?;
        }

        for (_, (mut mesh_comp, ), _)
        in <(MeshComponent, )>::query_deep_mut(scene) {
            if mesh_comp.vao.is_some() { continue; };

            let mesh = Mesh::load(path::as_fs_abs(&state.project_root, &mesh_comp.mesh))?;
            let mat = Material::load(path::as_fs_abs(&state.project_root, &mesh_comp.mat))?;

            let vao = Vao::from(&mesh, &mat)?;

            mesh_comp.vao = Some(vao);
        }

        // Now we can properly render them
        for (entity, (mesh_comp, ), _)
        in <(MeshComponent, )>::query_deep(scene) {
            let vao = mesh_comp.vao.as_ref().unwrap();

            state.shader.enable();
            state.shader.set_mat4("model", &(combined_transform(scene, entity) * ctx.transform));

            let CameraMats{view_mat, projection_mat} = state.camera_mats.as_ref().unwrap();

            state.shader.set_mat4("view", view_mat);
            state.shader.set_mat4("projection", projection_mat);

            vao.draw();
        }

        Ok(())
    }

    fn compute_camera_mats(&self) -> Option<CameraMats> {
        let scene = self.scene.as_ref().unwrap();

        // TODO Search for cameras in downstream scenes
        <(CameraComponent, )>::query_shallow(scene).next().map(|(entity, _, _)| {
            let transform = combined_transform(scene, entity);

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
                    let [width, height] = self.state.canvas_size;
                    let aspect_ratio = width as f32 / height as f32;
                    Mat4::perspective_rh_gl(90_f32.to_radians(), aspect_ratio, 0.1, 100.0)
                },
            }
        })
    }
}

fn combined_transform(scene: &Scene, mut entity: Entity) -> Mat4 {
    let mut transform_components = Vec::new();

    loop {
        let transform_component = scene.get_one::<TransformComponent>(entity)
            .expect("entity does not have a transform component");
        transform_components.push(transform_component);

        let hierarchy_component = scene.get_one::<HierarchyComponent>(entity)
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

fn clear_canvas() {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
