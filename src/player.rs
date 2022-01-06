use crate::camera;
use crate::input;
use crate::loader;

use glium::glutin;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub velocity: (f32,f32,f32),

    pub lin_speed: f32,
    pub rot_speed: f32,
    pub jump_power: f32,
    falling: bool,
    pub camera: camera::Camera,
}

impl Player {
    pub fn new(
        (x, y, z): (f32, f32, f32),
        lin_speed: f32,
        rot_speed: f32,
        jump_power: f32,
        camera: camera::Camera,
    ) -> Player {
        Player {
            x,
            y,
            z,
            velocity: (0.0,0.0,0.0),
            lin_speed,
            rot_speed,
            jump_power,
            falling: true,
            camera,
        }
    }

    pub fn update(&mut self, delta: f32, keyboard_state: &input::Input, loader: &loader::ChunkLoader) {
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::W) {
            self.z += self.lin_speed * delta * self.camera.yaw.sin();
            self.x += self.lin_speed * delta * self.camera.yaw.cos();
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::S) {
            self.z -= self.lin_speed * delta * self.camera.yaw.sin();
            self.x -= self.lin_speed * delta * self.camera.yaw.cos();
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::A) {
            self.z += self.lin_speed * delta * self.camera.yaw.cos();
            self.x -= self.lin_speed * delta * self.camera.yaw.sin();
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::D) {
            self.z -= self.lin_speed * delta * self.camera.yaw.cos();
            self.x += self.lin_speed * delta * self.camera.yaw.sin();
        }

        // Change to gravity once chunks are implemented
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::Space) {
            if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::LShift) {
                self.y -= self.jump_power * delta;
            } else {
                self.y += self.jump_power * delta;
            }
        }

        // if let Some (block) = loader.get_block((self.x.floor() as i32, self.y.floor() as i32, self.z.floor() as i32)) {
        //     if block.id != 0 {
        //         self.falling = false;
        //         self.velocity.1 = 0.0;
        //     }
        // }

        //if self.falling && loader.get_chunk((self.x.floor() as f32 / crate::chunk::CHUNK_SIZE.0 as f32) as i32, self.y.floor() as i32 / crate::chunk::CHUNK_SIZE.1 as i32, self.z.floor() as i32 / crate::chunk::CHUNK_SIZE.2 as i32)).is_some() {
            //self.velocity.1 -= 10.0 * delta;
        //}

        self.x += self.velocity.0 * delta;
        self.y += self.velocity.1 * delta;
        self.z += self.velocity.2 * delta;

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

    fn collide (&mut self, loader: &loader::ChunkLoader) {

    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            velocity: (0.0,0.0,0.0),
            lin_speed: 10.0,
            rot_speed: 1.0,
            jump_power: 10.0,
            falling: true,
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
