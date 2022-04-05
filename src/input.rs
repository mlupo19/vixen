use std::collections::HashMap;

use glium::glutin::event::{ElementState, MouseButton, VirtualKeyCode};

/// Keeps track of which keys have been pressed.
pub struct Input {
    state: HashMap<VirtualKeyCode, ElementState>,
    mouse_button_state: HashMap<MouseButton, ElementState>,
    mouse_delta: (f64, f64),
}
impl Input {
    /// Constructs a new KeyboardState with all the keys released.
    pub fn new() -> Input {
        Input {
            state: HashMap::new(),
            mouse_button_state: HashMap::new(),
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
    pub fn process_keyboard_event(&mut self, key_state: ElementState, code: VirtualKeyCode) {
        match key_state {
            ElementState::Pressed => {
                self.state.insert(code, key_state);
            }
            ElementState::Released => {
                self.state.remove(&code);
            }
        }
    }

    /// Returns true if 'button' is pressed for any device
    pub fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_button_state
            .get(button)
            .map(|&s| s == ElementState::Pressed)
            .unwrap_or(false)
    }

    /// Returns false if 'button' is released from all devices
    pub fn is_mouse_button_released(&self, button: &MouseButton) -> bool {
        !self.is_mouse_button_pressed(button)
    }

    pub fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        self.mouse_button_state.insert(button, state);
    }

    pub fn update_mouse_motion(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    }

    pub fn get_mouse_delta_x(&self) -> f64 {
        self.mouse_delta.0
    }

    pub fn get_mouse_delta_y(&self) -> f64 {
        self.mouse_delta.1
    }
}
