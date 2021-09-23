use gl;

pub struct Framebuffer {
    framebuffer_id: u32,
    texture_id: u32,
    depth_n_stencil_id: u32,
}

impl Framebuffer {
    pub fn new(size: (i32, i32)) -> Framebuffer {
        let mut framebuffer_id = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer_id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
        }

        let (width, height) = size;

        let mut texture_id = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE, 0 as _);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
        }

        let mut depth_n_stencil_id = 0;

        unsafe {
            gl::GenRenderbuffers(1, &mut depth_n_stencil_id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth_n_stencil_id);

            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);

            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_n_stencil_id);
        }

        unsafe {
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            framebuffer_id,
            texture_id,
            depth_n_stencil_id,
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id); }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn get_tex_id(&self) -> u32 {
        self.texture_id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.framebuffer_id);
            gl::DeleteTextures(1, &self.texture_id);
            gl::DeleteRenderbuffers(1, &self.depth_n_stencil_id);
        }
    }
}
