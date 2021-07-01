use derivative::Derivative;
use glam::Vec3;

use crate::component::Component;
use crate::system::System;

#[derive(Debug)]
pub struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Entity {
    transform: Transform,
    children: Vec<Entity>,
    #[derivative(Debug = "ignore")]
    components: Vec<Component>,
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            transform: Transform {
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(0.0, 0.0, 0.0),
            },
            children: Vec::new(),
            components: Vec::new(),
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
