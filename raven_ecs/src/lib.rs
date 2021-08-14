#![feature(try_blocks)]
#![feature(cell_filter_map)]
#![feature(label_break_value)]
#![feature(type_alias_impl_trait)]
#![feature(stmt_expr_attributes)]

use serde::{Deserialize, Serialize};

pub use world::query;
use serde::de::DeserializeOwned;

#[macro_export]
macro_rules! deref_vec {
    ($e:expr) => {
        $e.iter().map(|e| e.deref()).collect::<Vec<_>>()
    }
}

mod pool;

pub mod world;

type ID = usize;
type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Entity {
    id: ID,
    version: Version,
}

pub trait Component: 'static + Sized + Serialize + DeserializeOwned {}

impl<T: 'static + Serialize + DeserializeOwned> Component for T {}
