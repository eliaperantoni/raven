use std::collections::HashSet;

use glutin::event::VirtualKeyCode;

pub struct InputManager {
    state: HashSet<VirtualKeyCode>,
}

impl Default for InputManager {
    fn default() -> Self {
        InputManager {
            state: HashSet::default()
        }
    }
}

impl InputManager {
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state.contains(&key)
    }

    pub fn set_pressed(&mut self, key: VirtualKeyCode, pressed: bool) {
        if pressed {
            self.state.insert(key);
        } else {
            self.state.remove(&key);
        }
    }
}
