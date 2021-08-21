use crate::resource::{Material, Mesh};
use crate::Result;

use std::mem;

use gl;

// 3 for the position
// 3 for the normal
// 2 for the UV
const FLOATS_PER_VERT: usize = 3 + 3 + 2;

#[derive(Debug)]
pub struct Vao {
    id: u32,
    n_indices: usize,
}

impl Drop for Vao {
    fn drop(&mut self) {
        dbg!("dropping vao");
    }
}

impl Vao {
    pub fn from(mesh: &Mesh, mat: &Material) -> Result<Vao> {
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
        let attr_vec = build_vert_attr_vec(mesh);
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (FLOATS_PER_VERT * attr_vec.len() * mem::size_of::<f32>()) as _,
                attr_vec.as_ptr() as _,
                gl::STATIC_DRAW,
            );
        }

        // How many bytes each vertex takes up
        let stride = FLOATS_PER_VERT * mem::size_of::<f32>();

        // Define and enable the following vertex attribute pointers:
        //   0 => [f32; 3]: Position
        //   1 => [f32; 3]: Normal
        //   2 => [f32; 2]: UV coordinates
        unsafe {
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride as _,
                (0 * mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexArrayAttrib(vao_id, 0);

            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride as _,
                (3 * mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexArrayAttrib(vao_id, 1);

            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride as _,
                (6 * mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexArrayAttrib(vao_id, 2);
        }

        let mut ebo_id: u32 = 0;

        // Setup EBO
        unsafe {
            gl::GenBuffers(1, &mut ebo_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo_id);
        }

        let indices_vec = build_indices_vec(mesh);
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices_vec.len() * mem::size_of::<u32>()) as _,
                indices_vec.as_ptr() as _,
                gl::STATIC_DRAW,
            );
        }

        // TODO Load textures

        // Unbind buffers
        unsafe {
            gl::BindVertexArray(0);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Ok(Vao {
            id: vao_id,
            n_indices: indices_vec.len(),
        })
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawElements(
                gl::TRIANGLES,
                self.n_indices as _,
                gl::UNSIGNED_INT,
                0 as _,
            );
            gl::BindVertexArray(0);
        }
    }
}

fn build_vert_attr_vec(mesh: &Mesh) -> Vec<f32> {
    let mut buf: Vec<f32> = Vec::with_capacity(FLOATS_PER_VERT * mesh.vertices.len());

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

    buf
}

fn build_indices_vec(mesh: &Mesh) -> Vec<u32> {
    let mut buf = Vec::with_capacity(mesh.indices.len());

    for index in &mesh.indices {
        buf.push(*index);
    }

    buf
}
