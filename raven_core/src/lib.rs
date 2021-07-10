use std::error::Error;

use entity::Entity;
use system::camera::CameraSystem;
use system::renderer::RendererSystem;

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
pub mod framebuffer;

pub struct Raven {
    renderer_sys: RendererSystem,
    camera_sys: CameraSystem,
}

impl Raven {
    pub fn new() -> Result<Raven, Box<dyn Error>> {
        Ok(Raven {
            renderer_sys: RendererSystem::new()?,
            camera_sys: CameraSystem::default(),
        })
    }

    pub fn do_frame(&mut self, scene: &mut Entity) {
        self.renderer_sys.clear();

        scene.accept(&mut self.camera_sys);

        self.renderer_sys.update_matrices(&self.camera_sys);
        scene.accept(&mut self.renderer_sys);
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        let [width, height] = size;

        unsafe {
            gl::Viewport(0, 0, width as _, height as _);
        }

        self.camera_sys.aspect_ratio = width / height;
    }
}
