use std::ffi::CString;
use std::ptr;

use gl;
use glam::Mat4;

use crate::Result;

pub struct Shader {
    id: u32,
    components: Vec<ShaderComponent>,
}

impl Shader {
    pub fn new() -> Shader {
        Shader {
            id: unsafe { gl::CreateProgram() },
            components: Vec::new(),
        }
    }

    pub fn with_component(mut self, comp: ShaderComponent) -> Shader {
        unsafe {
            gl::AttachShader(self.id, comp.id);
        }
        self.components.push(comp);
        self
    }

    pub fn build(self) -> Result<Shader> {
        use gl::types::{GLint, GLchar};

        unsafe {
            gl::LinkProgram(self.id);

            let mut link_status = gl::FALSE as GLint;
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut link_status);

            if link_status != (gl::TRUE as GLint) {
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

                return Err(Box::from(std::str::from_utf8(&buf)?));
            }
        }

        Ok(self)
    }

    pub fn enable(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_mat4<T: AsRef<str>>(&self, name: T, mat: &Mat4) {
        unsafe {
            let s = CString::new(name.as_ref()).unwrap();
            let loc = gl::GetUniformLocation(self.id, s.as_ptr());
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.as_ref() as _);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub enum ShaderComponentType {
    VERTEX,
    FRAGMENT,
}

impl From<ShaderComponentType> for gl::types::GLenum {
    fn from(t: ShaderComponentType) -> Self {
        match t {
            ShaderComponentType::VERTEX => gl::VERTEX_SHADER,
            ShaderComponentType::FRAGMENT => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct ShaderComponent {
    id: u32,
}

impl ShaderComponent {
    pub fn new<P: AsRef<str>>(source: P, t: ShaderComponentType) -> Result<ShaderComponent> {
        use gl::types::{GLint, GLchar};

        // Convert to C string
        let source = CString::new(source.as_ref().as_bytes()).unwrap();

        let id = unsafe {
            // Create shader and load source
            let id = gl::CreateShader(t.into());
            gl::ShaderSource(id, 1, &source.as_ptr(), ptr::null());

            // Compile shader
            let mut compile_status = gl::FALSE as GLint;

            gl::CompileShader(id);
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compile_status);

            if compile_status != (gl::TRUE as GLint) {
                let mut log_len = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut log_len);

                let mut buf = Vec::with_capacity(log_len as usize);
                buf.set_len((log_len as usize) - 1);

                gl::GetShaderInfoLog(
                    id,
                    log_len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                return Err(Box::from(std::str::from_utf8(&buf)?));
            }

            id
        };

        Ok(ShaderComponent { id })
    }
}

impl Drop for ShaderComponent {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
