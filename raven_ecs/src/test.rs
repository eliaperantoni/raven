use serde::{Deserialize, Serialize};

use crate::{Component, Entity};
use crate::world::World;

macro_rules! deref_vec {
    ($e:expr) => {
        $e.iter().map(|e| e.deref()).collect::<Vec<_>>()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CompX {
    pub f: String,
}

impl CompX {
    pub fn new(s: &str) -> CompX {
        CompX {
            f: s.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CompY {
    pub f: String,
}

impl CompY {
    pub fn new(s: &str) -> CompY {
        CompY {
            f: s.to_string(),
        }
    }
}

#[::typetag::serde]
impl Component for CompX {
    fn inject(self: Box<Self>, w: &mut World, e: Entity) {
        w.attach::<Self>(e, *self);
    }
}

#[::typetag::serde]
impl Component for CompY {
    fn inject(self: Box<Self>, w: &mut World, e: Entity) {
        w.attach::<Self>(e, *self);
    }
}
