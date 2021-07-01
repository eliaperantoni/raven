use crate::entity::Entity;

pub mod camera;

pub trait System {
    fn visit_entity(&mut self, entity: &mut Entity);
}
