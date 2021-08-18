#![feature(try_blocks)]
#![feature(cell_filter_map)]
#![feature(label_break_value)]
#![feature(type_alias_impl_trait)]
#![feature(stmt_expr_attributes)]

#[doc(hidden)]
pub use typetag;

#[doc(hidden)]
pub use raven_ecs_proc::Component;
pub use world::query::Query;
pub use world::World;

#[cfg(test)]
#[macro_use]
mod test;

mod pool;
mod world;

type ID = usize;
type Version = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Entity {
    id: ID,
    version: Version,
}

#[typetag::serde(tag = "type")]
pub trait Component: 'static {
    fn inject(self: Box<Self>, w: &mut World, e: Entity);
}
