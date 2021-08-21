#![feature(with_options)]

use std::collections::HashMap;
use std::path::{PathBuf, Path};

pub use ::glam;
use gl::{self, types::*};

pub use ::raven_ecs::Entity;

use crate::resource::{Resource, Scene};
use std::error::Error;

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

    fn get_or_load<P: AsRef<Path>>(&mut self, path: P) -> Result<Resource> {
        todo!()
    }

    fn do_frame(&mut self) -> Result<()> {
        todo!()
    }
}
