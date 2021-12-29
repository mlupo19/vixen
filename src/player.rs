use crate::camera;
use crate::input;

use glium::glutin;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub lin_speed: f32,
    pub rot_speed: f32,
    pub camera: camera::Camera,
}

impl Player {
    pub fn new(
        (x, y, z): (f32, f32, f32),
        lin_speed: f32,
        rot_speed: f32,
        camera: camera::Camera,
    ) -> Player {
        Player {
            x: x,
            y: y,
            z: z,
            lin_speed: lin_speed,
            rot_speed: rot_speed,
            camera: camera,
        }
    }

    pub fn update(&mut self, delta: f32, keyboard_state: &input::KeyboardState) {
        if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::W) {
            self.z += self.lin_speed * delta * self.camera.yaw.sin();
            self.x += self.lin_speed * delta * self.camera.yaw.cos();
        }
        if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::S) {
            self.z -= self.lin_speed * delta * self.camera.yaw.sin();
            self.x -= self.lin_speed * delta * self.camera.yaw.cos();
        }
        if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::A) {
            self.z += self.lin_speed * delta * self.camera.yaw.cos();
            self.x -= self.lin_speed * delta * self.camera.yaw.sin();
        }
        if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::D) {
            self.z -= self.lin_speed * delta * self.camera.yaw.cos();
            self.x += self.lin_speed * delta * self.camera.yaw.sin();
        }

        // Change to gravity once chunks are implemented
        if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::Space) {
            if keyboard_state.is_pressed(&glutin::event::VirtualKeyCode::LShift) {
                self.y -= self.lin_speed * delta;
            } else {
                self.y += self.lin_speed * delta;
            }
        }

        self.camera.x = self.x;
        self.camera.y = self.y + 2.0;
        self.camera.z = self.z;
    }

    pub fn get_camera(&self) -> &camera::Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut camera::Camera {
        &mut self.camera
    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            lin_speed: 10.0,
            rot_speed: 1.0,
            camera: camera::Camera {
                x: 0.0,
                y: 0.0,
                z: -1.5,
                pitch: 3.141592 / 2.0,
                yaw: 3.141592 / 2.0,
                roll: 0.0,
            },
        }
    }
}
