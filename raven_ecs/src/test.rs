use serde::{Deserialize, Serialize};
use typetag;

use crate::Component;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CompX {
    f: String,
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
    f: String,
}

impl CompY {
    pub fn new(s: &str) -> CompY {
        CompY {
            f: s.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CompZ {
    f: String,
}

impl CompZ {
    pub fn new(s: &str) -> CompZ {
        CompZ {
            f: s.to_string(),
        }
    }
}

#[typetag::serde]
impl Component for CompX {}

#[typetag::serde]
impl Component for CompY {}

#[typetag::serde]
impl Component for CompZ {}
