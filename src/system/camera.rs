use super::System;
use crate::entity::Entity;

pub struct Camera {

}

impl System for Camera {
    fn visit_entity(&mut self, entity: &mut Entity) {
        dbg!(entity);
    }
}
