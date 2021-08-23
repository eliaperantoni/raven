#![feature(with_options)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use gl::{self, types::*};
pub use glam;
use glam::Mat4;

use ecs::*;

use crate::component::{HierarchyComponent, MeshComponent, TransformComponent};
use crate::io::Serializable;
use crate::resource::{Scene, Mesh, Material};
use crate::vao::Vao;
use crate::shader::Shader;
use crate::standard_shader::get_standard_shader;

pub mod ecs {
    pub use raven_ecs::*;
}

pub mod resource;
pub mod component;
pub mod io;

mod vao;
mod shader;
mod standard_shader;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub struct Processor {
    scene: Scene,
    shader: Shader,

    view_mat: Mat4,
    proj_mat: Mat4,
}

impl Processor {
    pub fn new(scene: Scene) -> Result<Processor> {
        Ok(Processor {
            scene,
            shader: get_standard_shader()?,
            view_mat: Mat4::default(),
            proj_mat: Mat4::default(),
        })
    }

    fn clear_canvas(&self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn combined_transform(&self, mut entity: Entity) -> Mat4 {
        let mut transform_components = Vec::new();

        loop {
            let transform_component = self.scene.get_one::<TransformComponent>(entity)
                .expect("entity does not have a transform component");
            transform_components.push(transform_component);

            let hierarchy_component = self.scene.get_one::<HierarchyComponent>(entity)
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

    fn do_frame(&mut self) -> Result<()> {
        self.clear_canvas();

        // First of all, we need to initialize a VAO for each MeshComponent that we haven't seen yet
        for (_, (mut mesh_comp,), _)
        in <(MeshComponent, )>::query_deep_mut(&mut self.scene) {
            if mesh_comp.vao.is_some() { continue; };

            let mesh = Mesh::load(&mesh_comp.mesh)?;
            let mat = Material::load(&mesh_comp.mat)?;

            let vao = Vao::from(&mesh, &mat)?;

            mesh_comp.vao = Some(vao);
        }

        // Now we can properly render them
        for (entity, (mesh_comp,), _)
        in <(MeshComponent, )>::query_deep(&self.scene) {
            self.clear_canvas();

            let vao = mesh_comp.vao.as_ref().unwrap();

            self.shader.enable();
            self.shader.set_mat4("model", self.combined_transform(entity));
            self.shader.set_mat4("view", self.view_mat);
            self.shader.set_mat4("projection", self.proj_mat);

            vao.draw();
        }

        Ok(())
    }
}
