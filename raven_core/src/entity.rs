use glam::Mat4;

use crate::id::ID;

pub struct Entity {
    pub transform: Mat4,
    pub parent: Option<ID>,
    pub children: Vec<ID>,
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            transform: Mat4::IDENTITY,
            parent: None,
            children: Vec::new(),
        }
    }
}
