use crate::entity::Entity;

pub mod camera;
pub mod renderer;

pub trait System {
    fn visit_entity(&mut self, _entity: &mut Entity) {}
}
