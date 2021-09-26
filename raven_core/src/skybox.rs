use std::error::Error;
use std::mem;
use std::ptr;

use crate::resource::Texture;
use crate::shader::{Shader, ShaderComponent, ShaderComponentType};
use crate::CameraMats;
use crate::glam::{Mat3, Mat4};
use glam::Affine3A;

const SKY_RIGHT: &'static [u8] = include_bytes!("../skybox/right.tex");
const SKY_LEFT: &'static [u8] = include_bytes!("../skybox/left.tex");
const SKY_TOP: &'static [u8] = include_bytes!("../skybox/top.tex");
const SKY_BOTTOM: &'static [u8] = include_bytes!("../skybox/bottom.tex");
const SKY_FRONT: &'static [u8] = include_bytes!("../skybox/front.tex");
const SKY_BACK: &'static [u8] = include_bytes!("../skybox/back.tex");

const CUBE_VERTICES: &[f32] = &[
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

    1.0, -1.0, -1.0,
    1.0, -1.0,  1.0,
    1.0,  1.0,  1.0,
    1.0,  1.0,  1.0,
    1.0,  1.0, -1.0,
    1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
    1.0,  1.0,  1.0,
    1.0,  1.0,  1.0,
    1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
    1.0,  1.0, -1.0,
    1.0,  1.0,  1.0,
    1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
    1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
    1.0, -1.0,  1.0,
];

pub(crate) struct Skybox {
    vao_id: u32,
    vbo_id: u32,
    texture_id: u32,

    shader: Shader,
}

impl Skybox {
    pub(crate) fn load() -> Result<Skybox, Box<dyn Error>> {
        let mut vao_id: u32 = 0;

        // Setup VAO
        unsafe {
            gl::GenVertexArrays(1, &mut vao_id);
            gl::BindVertexArray(vao_id);
        }

        let mut vbo_id: u32 = 0;

        // Setup VBO
        unsafe {
            gl::GenBuffers(1, &mut vbo_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);
        }

        // Loads vertex data
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (CUBE_VERTICES.len() * mem::size_of::<f32>()) as _,
                CUBE_VERTICES.as_ptr() as _,
                gl::STATIC_DRAW,
            );
        }

        // Define and enable the following vertex attribute pointers:
        //   0 => [f32; 3]: Position
        unsafe {
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * mem::size_of::<f32>()) as _,
                ptr::null(),
            );
            gl::EnableVertexArrayAttrib(vao_id, 0);
        }

        // Unbind
        unsafe {
            gl::BindVertexArray(0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let right = bincode::deserialize::<Texture>(SKY_RIGHT)?;
        let left = bincode::deserialize::<Texture>(SKY_LEFT)?;
        let top = bincode::deserialize::<Texture>(SKY_TOP)?;
        let bottom = bincode::deserialize::<Texture>(SKY_BOTTOM)?;
        let front = bincode::deserialize::<Texture>(SKY_FRONT)?;
        let back = bincode::deserialize::<Texture>(SKY_BACK)?;

        let mut texture_id = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);

            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl::RGBA as _, right.size[0] as _, right.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, right.raw.as_ptr() as _);
            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl::RGBA as _, left.size[0] as _, left.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, left.raw.as_ptr() as _);
            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl::RGBA as _, top.size[0] as _, top.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, top.raw.as_ptr() as _);
            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl::RGBA as _, bottom.size[0] as _, bottom.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, bottom.raw.as_ptr() as _);
            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl::RGBA as _, front.size[0] as _, front.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, front.raw.as_ptr() as _);
            gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl::RGBA as _, back.size[0] as _, back.size[1] as _, 0,
                           gl::RGBA, gl::UNSIGNED_BYTE, back.raw.as_ptr() as _);

            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as _);

            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
        }

        let shader = get_skybox_shader()?;

        Ok(Skybox {
            vao_id,
            vbo_id,
            texture_id,
            shader,
        })
    }

    pub fn draw(&mut self, camera_mats: &CameraMats) {
        unsafe {
            gl::DepthMask(gl::FALSE);
        }

        self.shader.enable();

        let view_mat_3 = Mat3::from(camera_mats.view_mat.clone());
        let view_mat_4 = Mat4::from(Affine3A::from_mat3(view_mat_3));

        self.shader.set_mat4("view", &view_mat_4);
        self.shader.set_mat4("projection", &camera_mats.projection_mat);

        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture_id);

            gl::BindVertexArray(self.vao_id);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);

            gl::DepthMask(gl::TRUE);
        }
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao_id);
            gl::DeleteBuffers(1, &self.vbo_id);
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}

fn get_skybox_shader() -> Result<Shader, Box<dyn Error>> {
    Shader::new()
        .with_component(ShaderComponent::new(SKYBOX_VERT_SHADER, ShaderComponentType::VERTEX)?)
        .with_component(ShaderComponent::new(SKYBOX_FRAG_SHADER, ShaderComponentType::FRAGMENT)?)
        .build()
}

const SKYBOX_VERT_SHADER: &'static str = r"
#version 330 core
layout (location = 0) in vec3 pos_in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 frag_pos;

void main() {
    gl_Position = projection * view * vec4(pos_in, 1.0);

    frag_pos = pos_in;
}
";

const SKYBOX_FRAG_SHADER: &'static str = r"
#version 330 core

in vec3 frag_pos;

out vec4 color;

uniform samplerCube sampler;

void main() {
    color = texture(sampler, frag_pos);
}
";
