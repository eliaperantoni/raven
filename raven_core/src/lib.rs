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
    scene: Entity,
    renderer_sys: RendererSystem,
    camera_sys: CameraSystem,
}

impl Raven {
    pub fn from_scene(scene: Entity) -> Result<Raven, Box<dyn Error>> {
        Ok(Raven {
            scene,
            renderer_sys: RendererSystem::new()?,
            camera_sys: CameraSystem::default(),
        })
    }

    pub fn do_frame(&mut self) {
        self.renderer_sys.clear();

        self.scene.accept(&mut self.camera_sys);

        self.renderer_sys.update_matrices(&self.camera_sys);
        self.scene.accept(&mut self.renderer_sys);
    }
}
