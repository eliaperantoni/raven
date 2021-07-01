use crate::entity::Entity;

pub mod camera;
pub mod renderer;

pub trait System {
    fn each_frame(&mut self) {}
    fn visit_entity(&mut self, _entity: &mut Entity) {}
}
