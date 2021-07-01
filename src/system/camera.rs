use crate::entity::Entity;

use super::System;

pub struct Camera {}

impl System for Camera {
    fn visit_entity(&mut self, entity: &mut Entity) {
        dbg!(entity);
    }
}
