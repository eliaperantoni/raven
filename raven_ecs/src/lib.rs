#![feature(try_blocks)]
#![feature(cell_filter_map)]
#![feature(label_break_value)]
#![feature(type_alias_impl_trait)]
#![feature(stmt_expr_attributes)]

use serde::{Deserialize, Serialize};
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

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    id: ID,
    version: Version,
}

#[typetag::serde(tag = "type")]
pub trait Component: 'static + Send {
    fn inject(self: Box<Self>, w: &mut World, e: Entity);
}
