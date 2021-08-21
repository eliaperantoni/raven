use crate::resource::{Material, Mesh};
use crate::Result;

use gl;

#[derive(Debug)]
pub struct Vao(pub(crate) u32);

impl Drop for Vao {
    fn drop(&mut self) {
        dbg!("dropping vao");
    }
}

impl Vao {
    pub fn from(mesh: &Mesh, mat: &Material) -> Result<Vao> {
        let mut id: u32 = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }

        Ok(Vao(id))
    }
}
