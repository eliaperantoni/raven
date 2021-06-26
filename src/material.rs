use std::error::Error;
use std::path::Path;
use image;
use super::texture::{self, Texture, TextureType};

mod russimp {
    pub use russimp::material::Material;
    pub use russimp::texture::TextureType;
}

pub struct Material {
    diffuse_tex: Option<Texture>,
    specular_tex: Option<Texture>,
}

pub fn from_russimp(mat: &russimp::Material, base_dir: &Path) -> Result<Material, Box<dyn Error>> {
    let load_first_tex_of_type = |t: TextureType| -> Result<Option<Texture>, Box<dyn Error>> {
        Ok(if let Some(tex) = mat.textures[&t.into()].first() {
            let mut tex_path = base_dir.to_path_buf();
            tex_path.push(&tex.path);

            Some(texture::from_path(&tex_path, t)?)
        } else {
            None
        })
    };

    Ok(Material{
        diffuse_tex: load_first_tex_of_type(TextureType::Diffuse)?,
        specular_tex: load_first_tex_of_type(TextureType::Specular)?,
    })
}
