use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::ptr;
use std::str;

use gl::{self, types::*};

pub enum ShaderType {
    VERTEX,
    FRAGMENT,
}

impl ShaderType {
    fn as_gl_enum(&self) -> GLenum {
        use ShaderType::*;
        match self {
            VERTEX => gl::VERTEX_SHADER,
            FRAGMENT => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader {
    path: String,
    t: ShaderType,
    id: u32,
}

impl Shader {
    pub fn new(t: ShaderType, path: &str) -> Shader {
        Shader {
            t,
            path: path.to_string(),
            id: 0,
        }
    }

    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        // Load source from file
        let source = fs::read_to_string(&self.path)?;
        // Convert to C string
        let source = CString::new(source.as_bytes()).unwrap();

        unsafe {
            // Create shader and load source
            self.id = gl::CreateShader(self.t.as_gl_enum());
            gl::ShaderSource(self.id, 1, &source.as_ptr(), ptr::null());

            // Compile shader
            let mut compile_status = gl::FALSE as GLint;

            gl::CompileShader(self.id);
            gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut compile_status);

            if compile_status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut log_len);

                let mut buf = Vec::with_capacity(log_len as usize);
                buf.set_len((log_len as usize) - 1);

                gl::GetShaderInfoLog(
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

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
