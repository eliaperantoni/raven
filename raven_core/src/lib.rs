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

pub mod ecs {
    pub use raven_ecs::*;
}

pub mod resource;
pub mod component;
pub mod io;

mod vao;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub struct Processor {
    scene: Scene,
}

impl Processor {
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

        for (entity, (mesh_comp,), (mesh_comp_n,))
        in <(MeshComponent, )>::query_deep(&self.scene) {
            let vao: &Vao = match mesh_comp.vao.as_ref() {
                Some(vao) => vao,
                None => {
                    let mesh = Mesh::load(&mesh_comp.mesh)?;
                    let mat = Material::load(&mesh_comp.mat)?;

                    let vao = Vao::from(&mesh, &mat)?;

                    self.scene.get_nth_mut::<MeshComponent>(entity, mesh_comp_n);

                    mesh_comp.vao = Some(vao);
                    mesh_comp.vao.as_ref().unwrap()
                },
            };

            // TODO Figure out a way to make this work you absolute ding dong
            let transform = self.combined_transform(entity);

            dbg!(vao);

            todo!()
        }

        todo!()
    }
}
