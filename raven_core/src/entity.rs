use derivative::Derivative;
use glam::{self, Quat, Vec3};

use crate::component::Component;
use crate::system::System;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Entity {
    pub transform: Transform,
    pub children: Vec<Entity>,
    #[derivative(Debug = "ignore")]
    pub components: Vec<Component>,
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            transform: Transform {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            children: Vec::default(),
            components: Vec::default(),
        }
    }
}

impl Entity {
    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    pub fn add_child(&mut self, child: Entity) {
        self.children.push(child);
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        for ele in &self.components {
            if let Some(ele) = ele.as_any().downcast_ref::<T>() {
                return Some(ele);
            }
        }
        None
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        for ele in &mut self.components {
            if let Some(ele) = ele.as_any_mut().downcast_mut::<T>() {
                return Some(ele);
            }
        }
        None
    }

    pub fn accept(&mut self, sys: &mut dyn System) {
        sys.visit_entity(self);
        for child in &mut self.children {
            child.accept(sys);
        }
    }
}
