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

mod shader;
mod shader_program;
mod model;
mod entity;
mod component;
mod system;
mod material;
mod texture;
mod mesh;
mod input;

fn main() {
    match main_err() {
        Err(e) => println!("{}", e),
        _ => ()
    }
}

const CAMERA_MOVE_SPEED: f32 = 10.0;
const CAMERA_LOOK_SPEED: f32 = 40.0;

fn main_err() -> Result<(), Box<dyn Error>> {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Raven");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let mut scene = build_demo_scene()?;

    let mut camera_sys = CameraSystem::default();
    camera_sys.aspect_ratio = {
        let physical_size = windowed_context.window().inner_size();
        physical_size.width as f32 / physical_size.height as f32
    };

    let mut renderer_sys = RendererSystem::new()?;

    let mut input_manager = InputManager::default();

    let mut last_frame = time::Instant::now();

    let mut yaw = 0f32;
    let mut pitch = 0f32;

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    unsafe {
                        gl::Viewport(0, 0, physical_size.width as i32, physical_size.height as i32)
                    };
                    camera_sys.aspect_ratio = physical_size.width as f32 / physical_size.height as f32;
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::Key(input) => {
                        if let Some(v_key) = input.virtual_keycode {
                            input_manager.set_key_pressed(v_key, input.state == ElementState::Pressed);
                        }
                    }
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        input_manager.set_mouse_motion((dx as f32, dy as f32));
                    }
                    _ => ()
                }
            }
            _ => (),
        }

        let now = time::Instant::now();
        let time_delta = now - last_frame;
        last_frame = now;

        if input_manager.is_key_pressed(VirtualKeyCode::Escape) {
            *control_flow = ControlFlow::Exit;
        }

        let mut input_vec = Vec3::default();

        if input_manager.is_key_pressed(VirtualKeyCode::W) {
            input_vec.z -= 1.0;
        }
        if input_manager.is_key_pressed(VirtualKeyCode::A) {
            input_vec.x -= 1.0;
        }
        if input_manager.is_key_pressed(VirtualKeyCode::S) {
            input_vec.z += 1.0;
        }
        if input_manager.is_key_pressed(VirtualKeyCode::D) {
            input_vec.x += 1.0;
        }

        let mut cam_entity = &mut scene.children[0];

        cam_entity.transform.position += cam_entity.transform.rotation.mul_vec3(
            input_vec.normalize_or_zero() * CAMERA_MOVE_SPEED * time_delta.as_secs_f32()
        );

        input_vec = Vec3::ZERO;

        if input_manager.is_key_pressed(VirtualKeyCode::Q) {
            input_vec.y -= 1.0;
        }
        if input_manager.is_key_pressed(VirtualKeyCode::E) {
            input_vec.y += 1.0;
        }

        cam_entity.transform.position += input_vec.normalize_or_zero() * CAMERA_MOVE_SPEED * time_delta.as_secs_f32();

        {
            let (mut dx, mut dy) = input_manager.get_mouse_motion();
            dx *= time_delta.as_secs_f32() * CAMERA_LOOK_SPEED * -1_f32;
            dy *= time_delta.as_secs_f32() * CAMERA_LOOK_SPEED * -1_f32;
            yaw += dx;
            pitch += dy;

            cam_entity.transform.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
        }

        renderer_sys.each_frame();

        scene.accept(&mut camera_sys);

        renderer_sys.update_matrices(&camera_sys);
        scene.accept(&mut renderer_sys);

        windowed_context.swap_buffers().unwrap();

        input_manager.set_mouse_motion((0.0, 0.0));
    });
}

fn build_demo_scene() -> Result<Entity, Box<dyn Error>> {
    let mut scene = Entity::default();
    scene.add_child(
        {
            let mut camera_entity = Entity::default();

            // camera_entity.transform.position.x += 3.0;
            // camera_entity.transform.position.y += 3.0;
            camera_entity.transform.position.z += 3.0;

            //  camera_entity.transform.rotation = Quat::from_euler(EulerRot::XYZ, 5.0, 0.0, -180_f32.to_radians());

            camera_entity.add_component(
                CameraComponent::default().into()
            );
            camera_entity
        },
    );
    scene.add_child(
        ModelLoader::from_file("models/backpack/backpack.obj")?
    );

    Ok(scene)
}
