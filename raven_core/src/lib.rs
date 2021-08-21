#![feature(with_options)]

use std::collections::HashMap;
use std::path::{PathBuf, Path};

pub use ::glam;
use gl::{self, types::*};

pub use ::raven_ecs::Entity;

use crate::resource::{Resource, Scene};
use std::error::Error;
use crate::io::Serializable;
use std::collections::hash_map::Entry;

pub mod resource;
pub mod component;
pub mod io;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub struct Processor {
    scene: Scene,
    storage: HashMap<PathBuf, Resource>,
}

impl Processor {
    fn clear(&self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn get_or_load<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Resource> {
        let key = PathBuf::from(path.as_ref());
        if !self.storage.contains_key(&key) {
            let resource = Resource::load(path.as_ref())?;
            self.storage.insert(key.clone(), resource);
        }

        Ok(self.storage.get_mut(&key).unwrap())
    }

    fn do_frame(&mut self) -> Result<()> {
        todo!()
    }
}
