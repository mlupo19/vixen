use std::mem::MaybeUninit;

use crate::camera;
use crate::input;
use crate::loader;

use glium::glutin;

use parry3d::bounding_volume::AABB;
use parry3d::bounding_volume::BoundingVolume;
use parry3d::na::Point3;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub velocity: (f32, f32, f32),

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
            velocity: (0.0, 0.0, 0.0),
            lin_speed,
            rot_speed,
            jump_power,
            falling: true,
            camera,
        }
    }

    pub fn update(
        &mut self,
        delta: f32,
        keyboard_state: &input::Input,
        loader: &loader::ChunkLoader,
    ) {
        let mut step = (0.0, 0.0, 0.0);

        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::W) {
            step.2 += self.lin_speed * self.camera.yaw.sin() * delta;
            step.0 += self.lin_speed * self.camera.yaw.cos() * delta;
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::S) {
            step.2 -= self.lin_speed * self.camera.yaw.sin() * delta;
            step.0 -= self.lin_speed * self.camera.yaw.cos() * delta;
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::A) {
            step.2 += self.lin_speed * self.camera.yaw.cos() * delta;
            step.0 -= self.lin_speed * self.camera.yaw.sin() * delta;
        }
        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::D) {
            step.2 -= self.lin_speed * self.camera.yaw.cos() * delta;
            step.0 += self.lin_speed * self.camera.yaw.sin() * delta;
        }

        if keyboard_state.is_key_pressed(&glutin::event::VirtualKeyCode::Space) {
            self.velocity.1 = self.jump_power;
        }

        self.velocity.1 -= 9.81 * delta;

        step.0 += self.velocity.0 * delta;
        step.1 += self.velocity.1 * delta;
        step.2 += self.velocity.2 * delta;

        step = self.collide(delta, loader, step);

        self.x += step.0;
        self.y += step.1;
        self.z += step.2;

        self.camera.x = self.x;
        self.camera.y = self.y + 1.5;
        self.camera.z = self.z;
    }

    pub fn get_camera(&self) -> &camera::Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut camera::Camera {
        &mut self.camera
    }

    fn collide(&mut self, delta: f32, loader: &loader::ChunkLoader, (dx, dy, dz): (f32, f32, f32)) -> (f32, f32, f32) {
        let (mut dx, mut dy, mut dz) = (dx, dy, dz);
        let (nx, ny, nz) = (self.x + dx, self.y + dy, self.z + dz);

        let player_box_current = create_player_aabb((self.x, self.y, self.z), (self.x, self.y, self.z));
        let player_box_stepped = create_player_aabb((self.x.min(nx), self.y.min(ny), self.z.min(nz)), (self.x.max(nx), self.y.max(ny), self.z.max(nz)));
        
        for x in (nx.floor() as i32 - 1)..(nx.floor() as i32 + 2) {
            for y in (ny.floor() as i32 - 1)..(ny.floor() as i32 + 3) {
                for z in (nz.floor() as i32 - 1)..(nz.floor() as i32 + 2) {
                    match loader.get_block([x,y,z]) {
                        None => {
                            self.velocity.1 = 0.0;
                            dy = 0.0;
                        },
                        Some(block) if block.id != 0 => {
                            let block_aabb = AABB::new(Point3::new(x as f32,y as f32,z as f32), Point3::new((x+1) as f32, (y+1) as f32, (z+1) as f32));
                            if player_box_stepped.intersects(&block_aabb) && !player_box_current.intersects(&block_aabb) {
                                
                                let x_box = create_player_aabb((self.x.min(nx), self.y, self.z), (self.x.max(nx), self.y, self.z));
                                let y_box = create_player_aabb((self.x, self.y.min(ny), self.z), (self.x, self.y.max(ny), self.z));
                                let z_box = create_player_aabb((self.x, self.y, self.z.min(nz)), (self.x, self.y, self.z.max(nz)));

                                if x_box.intersects(&block_aabb) {
                                    self.velocity.0 = 0.0;
                                    dx = 0.0;
                                }

                                if y_box.intersects(&block_aabb) {
                                    self.velocity.1 = 0.0;
                                    dy = 0.0;
                                }

                                if z_box.intersects(&block_aabb) {
                                    self.velocity.2 = 0.0;
                                    dz = 0.0;
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        (dx, dy, dz)
    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            velocity: (0.0, 0.0, 0.0),
            lin_speed: 10.0,
            rot_speed: 1.0,
            jump_power: 10.0,
            falling: true,
            camera: camera::Camera {
                x: 0.0,
                y: 100.0,
                z: 0.0,
                pitch: 3.141592 / 2.0,
                yaw: 0.0,
                roll: 0.0,
            },
        }
    }
}

const HALF_WIDTH: f32 = 0.25;
const HEIGHT: f32 = 1.5;
const HALF_DEPTH: f32 = 0.25;

#[inline]
fn create_player_aabb((x_min,y_min,z_min): (f32, f32, f32), (x_max,y_max,z_max): (f32, f32, f32)) -> AABB {
    AABB::new(Point3::new(x_min-HALF_WIDTH, y_min, z_min-HALF_DEPTH), Point3::new(x_max+HALF_WIDTH, y_max+HEIGHT, z_max+HALF_DEPTH))
}