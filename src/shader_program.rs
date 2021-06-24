use super::shader::{Shader};
use std::ptr;
use std::str;
use gl::{self, types::*};
use std::error::Error;

pub struct ShaderProgram {
    id: u32,

    vertex: Shader,
    fragment: Shader,
}

impl ShaderProgram {
    pub fn new(vertex: Shader, fragment: Shader) -> ShaderProgram {
        ShaderProgram {
            id: 0,

            vertex,
            fragment,
        }
    }

    pub fn link(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            self.id = gl::CreateProgram();

            gl::AttachShader(self.id, self.vertex.get_id());
            gl::AttachShader(self.id, self.fragment.get_id());

            gl::LinkProgram(self.id);

            let mut compile_status = gl::FALSE as GLint;
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut compile_status);

            if compile_status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut log_len);

                let mut buf = Vec::with_capacity(log_len as usize);
                buf.set_len((log_len as usize) - 1);

                gl::GetProgramInfoLog(
                    self.id,
                    log_len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                return Err(Box::from(str::from_utf8(&buf)?));
            }
        }

        Ok(())
    }

    pub fn enable(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}
