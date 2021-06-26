use std::error::Error;
use std::path::Path;
use image;

mod russimp {
    pub use russimp::material::Material;
    pub use russimp::texture::TextureType;
}

struct Texture {}

pub struct MaterialLoader {

}

pub struct Material {
    diffuse_tex: Vec<Texture>,
    specular_tex: Vec<Texture>,
}

impl MaterialLoader {
    pub fn from_russimp(mat: &russimp::Material, base_dir: &Path) -> Result<Material, Box<dyn Error>> {
        for tex in &mat.textures[&russimp::TextureType::Diffuse] {
            let mut tex_path = base_dir.to_path_buf();
            tex_path.push(&tex.path);


        }

        dbg!(mat);
        todo!()
    }
}
