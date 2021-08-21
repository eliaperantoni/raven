use std::error::Error;
use std::fs::File;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::resource::*;

pub trait Serializable: Sized {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>>;
    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>>;
}

fn save_bytes<T: Serialize, P: AsRef<Path>>(self_: &T, at: P) -> Result<(), Box<dyn Error>> {
    let file = File::with_options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(at)?;
    let writer = std::io::BufWriter::new(file);

    bincode::serialize_into(writer, self_).map_err(|err| Box::from(err))
}

fn save_text<T: Serialize, P: AsRef<Path>>(self_: &T, at: P) -> Result<(), Box<dyn Error>> {
    let file = File::with_options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(at)?;
    let writer = std::io::BufWriter::new(file);

    serde_json::to_writer(writer, self_).map_err(|err| Box::from(err))
}

fn load_bytes<T: DeserializeOwned, P: AsRef<Path>>(at: P) -> Result<T, Box<dyn Error>> {
    let file = File::with_options().read(true).open(at)?;
    let reader = std::io::BufReader::new(file);

    bincode::deserialize_from(reader).map_err(|err| Box::from(err))
}

fn load_text<T: DeserializeOwned, P: AsRef<Path>>(at: P) -> Result<T, Box<dyn Error>> {
    let file = File::with_options().read(true).open(at)?;
    let reader = std::io::BufReader::new(file);

    serde_json::from_reader(reader).map_err(|err| Box::from(err))
}

impl Serializable for Texture {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>> {
        save_bytes(self, at)
    }

    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>> {
        load_bytes(at)
    }
}

impl Serializable for Mesh {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>> {
        save_bytes(self, at)
    }

    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>> {
        load_bytes(at)
    }
}

impl Serializable for Material {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>> {
        save_text(self, at)
    }

    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>> {
        load_text(at)
    }
}

impl Serializable for Scene {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>> {
        save_text(self, at)
    }

    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>> {
        load_text(at)
    }
}

impl Serializable for Resource {
    fn save<P: AsRef<Path>>(&self, at: P) -> Result<(), Box<dyn Error>> {
        match self {
            Resource::Texture(tex) => tex.save(at),
            Resource::Material(mat) => mat.save(at),
            Resource::Mesh(mesh) => mesh.save(at),
            Resource::Scene(scene) => scene.save(at),
        }
    }

    fn load<P: AsRef<Path>>(at: P) -> Result<Self, Box<dyn Error>> {
        let ext = match at.as_ref().extension() {
            Some(os_ext) => os_ext.to_str(),
            None => return Err(Box::<dyn Error>::from("no extension")),
        };

        match ext {
            Some("tex") => Texture::load(at).map(|tex| Resource::Texture(tex)),
            Some("mat") => Material::load(at).map(|mat| Resource::Material(mat)),
            Some("mesh") => Mesh::load(at).map(|mesh| Resource::Mesh(mesh)),
            Some("scene") => Scene::load(at).map(|scene| Resource::Scene(scene)),
            _ => return Err(Box::<dyn Error>::from("unknown extension"))
        }
    }
}
