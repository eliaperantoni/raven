use std::error::Error;
use std::path::Path;

mod russimp {
    pub use russimp::texture::TextureType;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Diffuse,
    Specular,
}

impl Into<russimp::TextureType> for TextureType {
    fn into(self) -> russimp::TextureType {
        match self {
            TextureType::Diffuse => russimp::TextureType::Diffuse,
            TextureType::Specular => russimp::TextureType::Specular,
            _ => todo!()
        }
    }
}

pub struct Texture {
    pub t: TextureType,
    data: Vec<u8>,
}

pub fn from_path(path: &Path, t: TextureType) -> Result<Texture, Box<dyn Error>> {
    let tex = image::open(path)?.into_rgba8();
    Ok(Texture {
        t,
        data: tex.into_raw(),
    })
}
