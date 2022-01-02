use std::collections::HashMap;

use glium::glutin::event::{ElementState, VirtualKeyCode};

/// Keeps track of which keys have been pressed.
pub struct Input {
    state: HashMap<VirtualKeyCode, ElementState>,
    mouse_delta: (f64, f64),
}
impl Input {
    /// Constructs a new KeyboardState with all the keys released.
    pub fn new() -> Input {
        Input {
            state: HashMap::new(),
            mouse_delta: (0.0, 0.0),
        }
    }

    #[allow(dead_code)]
    /// Returns true if `key` is pressed.
    pub fn is_key_pressed(&self, key: &VirtualKeyCode) -> bool {
        self.state
            .get(key)
            .map(|&s| s == ElementState::Pressed)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    /// Returns true if `key` is released.
    pub fn is_key_released(&self, key: &VirtualKeyCode) -> bool {
        !self.is_key_pressed(key)
    }

    /// Processes a keyboard event and updated the internal state.
    pub fn process_event(&mut self, key_state: ElementState, code: VirtualKeyCode) {
        match key_state {
            ElementState::Pressed => {
                self.state.insert(code, key_state);
            }
            ElementState::Released => {
                self.state.remove(&code);
            }
        }
    }

    pub fn update_mouse(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    } 

    pub fn get_mouse_delta_x(&self) -> f64 {
        self.mouse_delta.0
    }

    pub fn get_mouse_delta_y(&self) -> f64 {
        self.mouse_delta.1
    }
}