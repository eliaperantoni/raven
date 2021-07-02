use std::cell::RefCell;
use std::error::Error;
use std::mem;
use std::ops::DerefMut;

use gl::{self, types::*};
use glam::{EulerRot, Quat};
use glutin::ContextBuilder;
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
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
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    match input {
                        KeyboardInput {
                            virtual_keycode: Some(virtual_keycode),
                            state,
                            ..
                        } => {
                            input_manager.set_pressed(virtual_keycode, state == ElementState::Pressed);
                        }
                        _ => ()
                    }
                }
                _ => (),
            },
            _ => (),
        }

        if input_manager.is_pressed(VirtualKeyCode::W) {
            scene.children[0].transform.position.z -= 0.0001;
        } else if input_manager.is_pressed(VirtualKeyCode::S) {
            scene.children[0].transform.position.z += 0.0001;
        }

        renderer_sys.each_frame();

        scene.accept(&mut camera_sys);

        renderer_sys.update_matrices(&camera_sys);
        scene.accept(&mut renderer_sys);

        windowed_context.swap_buffers().unwrap();
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
        ModelLoader::from_file("models/cube/cube.obj")?
    );

    Ok(scene)
}

const VERTEX_DATA: [GLfloat; 3 * (2 + 3)] = [
    0.5, -0.5, 1.0, 0.0, 0.0,
    -0.5, -0.5, 0.0, 1.0, 0.0,
    0.0, 0.5, 0.0, 0.0, 1.0,
];

fn setup() -> GLuint {
    let (mut vao, mut vbo) = (0, 0);

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0, 2, gl::FLOAT, gl::FALSE,
            (mem::size_of::<GLfloat>() * (2 + 3)) as GLsizei,
            (mem::size_of::<GLfloat>() * 0) as *const _,
        );
        gl::EnableVertexArrayAttrib(vao, 0);

        gl::VertexAttribPointer(
            1, 3, gl::FLOAT, gl::FALSE,
            (mem::size_of::<GLfloat>() * (2 + 3)) as GLsizei,
            (mem::size_of::<GLfloat>() * 2) as *const _,
        );
        gl::EnableVertexArrayAttrib(vao, 1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    return vao;
}

fn draw_frame(vao: GLuint) {
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        gl::BindVertexArray(vao);
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
}
