#![feature(with_options)]
#![feature(duration_constants)]

use std::error::Error;
use std::path::{Path, PathBuf};
use std::result::Result;

use gl;
pub use glam;
use glam::{Mat4, Quat, Vec3};
pub use mat4;
use mat4::decompose;

use ecs::*;

use crate::component::{CameraComponent, HierarchyComponent, MeshComponent, SceneComponent, TransformComponent};
use crate::io::Serializable;
use crate::resource::{Material, Mesh, Scene, Texture};
use crate::shader::Shader;
use crate::standard_shader::get_standard_shader;
use crate::vao::Vao;

use crate::skybox::Skybox;

pub mod ecs {
    pub use raven_ecs::*;
}

pub mod resource;
pub mod component;
pub mod io;
pub mod path;
pub mod framebuffer;
pub mod time;

mod vao;
mod tex;
mod shader;
mod standard_shader;
mod skybox;

pub struct Processor {
    state: ProcessorState,
    skybox: Skybox,
    scene: Option<Scene>,
}

pub(crate) struct ProcessorState {
    project_root: PathBuf,
    canvas_size: [u32; 2],
    shader: Shader,
    camera_mats: Option<CameraMats>,
}

struct CameraMats {
    view_mat: Mat4,
    projection_mat: Mat4,
}

#[derive(Debug)]
pub enum FrameError {
    NoCamera,
    Generic(Box<dyn Error>)
}

impl Processor {
    pub fn new<R: AsRef<Path>>(project_root: R) -> Result<Processor, Box<dyn Error>> {
        let skybox = Skybox::load()?;

        Ok(Processor {
            state: ProcessorState {
                project_root: project_root.as_ref().to_owned(),
                canvas_size: [800, 400],
                shader: get_standard_shader()?,
                camera_mats: None,
            },
            scene: None,
            skybox,
        })
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, scene_path: P) -> Result<(), Box<dyn Error>> {
        let scene_path = path::as_fs_abs(&self.state.project_root, scene_path);

        self.scene = Some(Scene::load(scene_path)?);
        Ok(())
    }

    pub fn get_scene(&self) -> Option<&Scene> {
        self.scene.as_ref()
    }

    pub fn get_scene_mut(&mut self) -> Option<&mut Scene> {
        self.scene.as_mut()
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.state.canvas_size = [width, height];
    }

    pub fn do_frame(&mut self) -> Result<(), FrameError> {
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

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }

        Processor::load_downstream_scenes(self.scene.as_mut().unwrap(), &self.state).map_err(|err| FrameError::Generic(err))?;

        self.state.camera_mats = Some(
            compute_camera_mats(self.scene.as_ref().unwrap(), Mat4::default(), &self.state.canvas_size)
            .ok_or_else(|| FrameError::NoCamera)?
        );

        self.skybox.draw(self.state.camera_mats.as_ref().unwrap());

        Processor::process_scene(self.scene.as_mut().unwrap(), &mut self.state, Mat4::default()).map_err(|err| FrameError::Generic(err))?;

        Ok(())
    }

    fn load_downstream_scenes(scene: &mut Scene, state: &ProcessorState) -> Result<(), Box<dyn Error>> {
        for (_, (mut scene_comp, ), _)
        in <(SceneComponent, )>::query_shallow_mut(scene) {
            // Ignore SceneComponents with no scene selected
            let scene = match scene_comp.scene.as_ref() {
                Some(scene) => scene,
                None => continue,
            };

            if scene_comp.loaded.is_none() {
                scene_comp.loaded = Some(Scene::load(path::as_fs_abs(&state.project_root, scene))?);
            }

            Processor::load_downstream_scenes(scene_comp.loaded.as_mut().unwrap(), state)?;
        }
        Ok(())
    }

    fn process_scene(scene: &mut Scene, state: &mut ProcessorState, base_transform: Mat4) -> Result<(), Box<dyn Error>> {
        let scene_containers: Vec<(Entity, Mat4)> = <(SceneComponent, )>::query_shallow(scene)
            .filter(|(_, (scene_comp,), _)| scene_comp.scene.is_some()) // Ignore SceneComponents with no scene selected
            .map(|(entity, _, _)| (entity, base_transform * combined_transform(scene, entity))).collect();

        for (entity, base_transform) in scene_containers {
            let mut scene_comp = scene.get_one_mut::<SceneComponent>(entity).unwrap();
            Processor::process_scene(scene_comp.loaded.as_mut().unwrap(), state, base_transform)?;
        }

        for (_, (mut mesh_comp, ), _)
        in <(MeshComponent, )>::query_deep_mut(scene) {
            if mesh_comp.vao.is_some() { continue; };

            let mesh = Mesh::load(path::as_fs_abs(&state.project_root, &mesh_comp.mesh))?;
            let mat = Material::load(path::as_fs_abs(&state.project_root, &mesh_comp.mat))?;

            let vao = Vao::from(&mesh)?;

            mesh_comp.vao = Some(vao);

            if let Some(tex_path) = mat.tex {
                let mut tex = Texture::load(path::as_fs_abs(&state.project_root, &tex_path))?;
                tex.load_gl();
                mesh_comp.tex = Some(tex);
            }
        }

        // Now we can properly render them
        for (entity, (mesh_comp, ), _)
        in <(MeshComponent, )>::query_deep(scene) {
            let vao = mesh_comp.vao.as_ref().unwrap();

            state.shader.enable();
            state.shader.set_mat4("model", &(base_transform * combined_transform(scene, entity)));

            let CameraMats { view_mat, projection_mat } = state.camera_mats.as_ref().unwrap();

            state.shader.set_mat4("view", view_mat);
            state.shader.set_mat4("projection", projection_mat);

            Texture::use_tex(mesh_comp.tex.as_ref(), &mut state.shader);

            vao.draw();
        }

        Ok(())
    }
}

fn compute_camera_mats(scene: &Scene, base_transform: Mat4, canvas_size: &[u32; 2]) -> Option<CameraMats> {
    for (_, (scene_comp, transform_comp), _)
    in <(SceneComponent, TransformComponent)>::query_deep(scene) {
        // Ignore SceneComponents with no scene selected
        if scene_comp.scene.is_none() {
            continue;
        }

        let transform = base_transform * transform_comp.0.clone();
        if let Some(mats) = compute_camera_mats(scene_comp.loaded.as_ref().unwrap(), transform, canvas_size) {
            return Some(mats);
        }
    }

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
                let [width, height] = canvas_size;
                let aspect_ratio = *width as f32 / *height as f32;
                Mat4::perspective_rh(90_f32.to_radians(), aspect_ratio, 0.1, 100.0)
            },
        }
    })
}

pub fn combined_transform(scene: &Scene, mut entity: Entity) -> Mat4 {
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

    for transform_component in transform_components.into_iter().rev() {
        out = out * transform_component.0;
    }

    out
}

fn clear_canvas() {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
