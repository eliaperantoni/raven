#![feature(try_blocks)]
#![feature(cell_filter_map)]
#![feature(label_break_value)]
#![feature(min_type_alias_impl_trait)]

mod pool;

pub mod world;

pub use world::query;

pub type ID = usize;
pub type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Entity {
    id: ID,
    version: Version,
}

pub trait Component: 'static + Sized {}

impl<T: 'static> Component for T {}
