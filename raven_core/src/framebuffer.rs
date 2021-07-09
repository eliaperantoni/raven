use gl;

pub struct Framebuffer {
    framebuffer_id: u32,
    texture_id: u32,
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

        unsafe {
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            framebuffer_id,
            texture_id,
        }
    }

    fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_id); }
    }

    fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn with<F: FnOnce()>(&self, do_fn: F) {
        self.bind();
        do_fn();
        self.unbind()
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
        }
    }
}
