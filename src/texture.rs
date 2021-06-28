use std::error::Error;
use std::path::Path;

mod russimp {
    pub use russimp::texture::TextureType;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Diffuse,
    Specular,
}

impl Into<russimp::TextureType> for TextureType {
    fn into(self) -> russimp::TextureType {
        match self {
            TextureType::Diffuse => russimp::TextureType::Diffuse,
            TextureType::Specular => russimp::TextureType::Specular,
        }
    }
}

// TODO Impl Debug trait without printing `data`
pub struct Texture {
    pub t: TextureType,
    data: Vec<u8>,
}

pub fn from_path(path: &Path, t: TextureType) -> Result<Texture, Box<dyn Error>> {
    let tex = image::open(path)?;
    let tex = tex.flipv();
    let tex = tex.into_rgba8();

    Ok(Texture {
        t,
        data: tex.into_raw(),
    })
}
