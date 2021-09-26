use crate::resource::Texture;
use crate::shader::Shader;

impl Texture {
    pub fn load_gl(&mut self) {
        let mut id: u32 = 0;

        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

            let [width, height] = self.size;

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _, width as _, height as _, 0, gl::RGBA, gl::UNSIGNED_BYTE, self.raw.as_ptr() as _);
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        self.id = Some(id);

        // Save some memory, it's already loaded in the GPU
        self.raw.clear();
    }

    pub fn use_tex(self_: Option<&Self>, shader: &mut Shader) {
        if let Some(tex) = self_ {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, tex.id.expect("texture not loaded"));
            }

            shader.set_bool("useSampler", true);
        } else {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            shader.set_bool("useSampler", false);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            unsafe {
                gl::DeleteTextures(1, &id);
            }
        }
    }
}
