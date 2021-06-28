pub mod camera;

use crate::entity::Entity;

pub trait System {
    fn visit_entity(&mut self, entity: &mut Entity);
}
