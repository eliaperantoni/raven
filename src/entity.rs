use glam::Vec3;

use crate::component::Component;

#[derive(Debug)]
pub struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

#[derive(Debug)]
pub struct Entity {
    transform: Transform,
    children: Vec<Entity>,
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
}
