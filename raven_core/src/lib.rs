use std::cell::RefCell;
use std::error::Error;
use std::mem;
use std::ops::DerefMut;
use std::time;

use gl::{self, types::*};
use glam::{EulerRot, Quat, Vec3};
use glutin::ContextBuilder;
use glutin::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

use component::CameraComponent;
use entity::Entity;
use input::InputManager;
use model::ModelLoader;
use shader::{Shader, ShaderType};
use shader_program::ShaderProgram;
use system::camera::CameraSystem;
use system::renderer::RendererSystem;

use crate::system::System;

pub mod shader;
pub mod shader_program;
pub mod model;
pub mod entity;
pub mod component;
pub mod system;
pub mod material;
pub mod texture;
pub mod mesh;
pub mod input;

pub struct Raven {
    scene: Entity,
    renderer_sys: RendererSystem,
    camera_sys: CameraSystem,
}

impl Raven {
    pub fn from_scene(scene: Entity) -> Result<Raven, Box<dyn Error>> {
        Ok(Raven {
            scene,
            renderer_sys: RendererSystem::new()?,
            camera_sys: {
                let mut camera_sys = CameraSystem::default();
                camera_sys
            },
        })
    }

    pub fn do_frame(&mut self) {
        self.renderer_sys.each_frame();

        self.scene.accept(&mut self.camera_sys);

        self.renderer_sys.update_matrices(&self.camera_sys);
        self.scene.accept(&mut self.renderer_sys);
    }
}
