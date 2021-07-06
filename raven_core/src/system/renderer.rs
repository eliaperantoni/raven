use std::error::Error;
use std::mem;

use gl;
use glam::Mat4;

use crate::component::MeshComponent;
use crate::entity::Entity;
use crate::mesh::Mesh;
use crate::shader::{Shader, ShaderType};
use crate::shader_program::ShaderProgram;

use super::camera::CameraSystem;
use super::System;

#[derive(Copy, Clone)]
struct Vao {
    vao: u32,
    indices_n: usize,
}

pub struct RendererSystem {
    vao: Option<Vao>,
    shader_program: ShaderProgram,

    view_mat: Mat4,
    proj_mat: Mat4,
}

impl RendererSystem {
    pub fn new() -> Result<RendererSystem, Box<dyn Error>> {
        let mut vertex_shader = Shader::new(ShaderType::VERTEX, "shaders/default/vertex.s");
        vertex_shader.load()?;

        let mut fragment_shader = Shader::new(ShaderType::FRAGMENT, "shaders/default/fragment.s");
        fragment_shader.load()?;

        let mut shader_program = ShaderProgram::new(vertex_shader, fragment_shader);
        shader_program.link()?;

        Ok(RendererSystem {
            vao: None,
            shader_program,

            view_mat: Mat4::default(),
            proj_mat: Mat4::default(),
        })
    }

    // Updates the view and projection matrices
    pub fn update_matrices(&mut self, cs: &CameraSystem) {
        self.view_mat = cs.get_view_mat();
        self.proj_mat = cs.get_proj_mat();
    }

    pub fn clear(&mut self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl System for RendererSystem {
    fn visit_entity(&mut self, entity: &mut Entity) {
        if let Some(mesh_c) = entity.get_component::<MeshComponent>() {
            let vao = if let Some(vao) = self.vao {
                vao
            } else {
                let vao = load_mesh(&mesh_c.mesh);
                self.vao = Some(vao);
                vao
            };

            self.shader_program.enable();
            self.shader_program.set_mat4("model", Mat4::IDENTITY);
            self.shader_program.set_mat4("view", self.view_mat);
            self.shader_program.set_mat4("projection", self.proj_mat);
            draw_vao(vao);
        }
    }
}

fn load_mesh(mesh: &Mesh) -> Vao {
    let mut vao = 0;

    // Setup VAO
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let mut vbo = 0;

    // Setup VBO
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    }

    // 3 for the position
    // 3 for the normal
    // 2 for the UV
    let floats_per_vert = 3 + 3 + 2;

    let mut buf: Vec<f32> = Vec::with_capacity(mesh.vertices.len() * floats_per_vert);

    for vert in &mesh.vertices {
        buf.push(vert.position.x);
        buf.push(vert.position.y);
        buf.push(vert.position.z);

        buf.push(vert.normal.x);
        buf.push(vert.normal.y);
        buf.push(vert.normal.z);

        buf.push(vert.uv.x);
        buf.push(vert.uv.y);
    }

    // Should be full now
    assert_eq!(buf.len(), buf.capacity());

    // Loads vertex data
    unsafe {
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (buf.len() * mem::size_of::<f32>()) as _,
            buf.as_ptr() as _,
            gl::STATIC_DRAW,
        );
    }

    let stride = (floats_per_vert * mem::size_of::<f32>()) as _;

    // Setup vertex attribute pointers
    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (0 * mem::size_of::<f32>()) as _,
        );
        gl::EnableVertexArrayAttrib(vao, 0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<f32>()) as _,
        );
        gl::EnableVertexArrayAttrib(vao, 1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<f32>()) as _,
        );
        gl::EnableVertexArrayAttrib(vao, 2);
    }

    let mut ebo = 0;

    // Setup EBO
    unsafe {
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    }

    let mut buf: Vec<u32> = Vec::with_capacity(mesh.indices.len());

    for ind in &mesh.indices {
        buf.push(*ind);
    }

    // Load indices
    unsafe {
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (buf.len() * mem::size_of::<u32>()) as _,
            buf.as_ptr() as _,
            gl::STATIC_DRAW,
        );
    }

    // Unbind buffers
    unsafe {
        gl::BindVertexArray(0);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    Vao {
        vao,
        indices_n: mesh.indices.len(),
    }
}

fn draw_vao(vao: Vao) {
    unsafe {
        gl::BindVertexArray(vao.vao as _);
        gl::DrawElements(
            gl::TRIANGLES,
            vao.indices_n as _,
            gl::UNSIGNED_INT,
            0 as _,
        );
        gl::BindVertexArray(0);
    }
}
