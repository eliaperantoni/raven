use std::collections::HashSet;

use glutin::event::VirtualKeyCode;

pub struct InputManager {
    keyboard_state: HashSet<VirtualKeyCode>,
    mouse_state: (f32, f32),
}

impl Default for InputManager {
    fn default() -> Self {
        InputManager {
            keyboard_state: HashSet::default(),
            mouse_state: (0.0, 0.0),
        }
    }
}

impl InputManager {
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keyboard_state.contains(&key)
    }

    pub fn get_mouse_motion(&self) -> (f32, f32) {
        self.mouse_state
    }

    pub fn set_key_pressed(&mut self, key: VirtualKeyCode, pressed: bool) {
        if pressed {
            self.keyboard_state.insert(key);
        } else {
            self.keyboard_state.remove(&key);
        }
    }

    pub fn set_mouse_motion(&mut self, motion: (f32, f32)) {
        self.mouse_state = motion;
    }
}
