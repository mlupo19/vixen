use std::time::Instant;

use crate::camera;
use crate::chunk::Block;
use crate::input;
use crate::loader;
use crate::loader::ChunkLoader;

use glium::glutin;

use parry3d::bounding_volume::BoundingVolume;
use parry3d::bounding_volume::AABB;
use parry3d::na::Point3;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub velocity: (f32, f32, f32),

    pub lin_speed: f32,
    pub rot_speed: f32,
    pub jump_power: f32,
    pub camera: camera::Camera,

    falling: bool,
    miner_builder: MinerBuilder,
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
            ..Default::default()
        }
    }

    pub fn update(&mut self, delta: f32, input: &input::Input, loader: &mut loader::ChunkLoader) {
        let mut step = (0.0, 0.0, 0.0);

        if input.is_key_pressed(&glutin::event::VirtualKeyCode::W) {
            step.2 += self.lin_speed * self.camera.yaw.sin() * delta;
            step.0 += self.lin_speed * self.camera.yaw.cos() * delta;
        }
        if input.is_key_pressed(&glutin::event::VirtualKeyCode::S) {
            step.2 -= self.lin_speed * self.camera.yaw.sin() * delta;
            step.0 -= self.lin_speed * self.camera.yaw.cos() * delta;
        }
        if input.is_key_pressed(&glutin::event::VirtualKeyCode::A) {
            step.2 += self.lin_speed * self.camera.yaw.cos() * delta;
            step.0 -= self.lin_speed * self.camera.yaw.sin() * delta;
        }
        if input.is_key_pressed(&glutin::event::VirtualKeyCode::D) {
            step.2 -= self.lin_speed * self.camera.yaw.cos() * delta;
            step.0 += self.lin_speed * self.camera.yaw.sin() * delta;
        }

        if input.is_key_pressed(&glutin::event::VirtualKeyCode::Space) && !self.falling {
            self.velocity.1 = self.jump_power;
            self.falling = true;
        }

        // Check if player is trying to mine
        if input.is_mouse_button_pressed(&glutin::event::MouseButton::Left) {
            let range = 4.0;
            let coord = cast_ray([self.camera.x,self.camera.y,self.camera.z], range, self.camera.pitch, self.camera.yaw, loader);
            mine(&mut self.miner_builder, coord, delta, 10.0, loader);
        }

        // Check if player is trying to build
        if input.is_mouse_button_pressed(&glutin::event::MouseButton::Right) {
            if self.miner_builder.can_build() {
                let range = 4.0;
                let coord = cast_ray_in_front([self.camera.x, self.camera.y, self.camera.z], range, self.camera.pitch, self.camera.yaw, loader);
                if let Some(coord) = coord {
                    if coord != [self.camera.x.floor() as i32, self.camera.y.floor() as i32, self.camera.z.floor() as i32]
                    && coord != [self.camera.x.floor() as i32, self.camera.y.floor() as i32 - 1, self.camera.z.floor() as i32] {
                        loader.set_block(coord, Block::new(1, 5.0));
                    }
                }
            }
        }

        self.velocity.1 -= 20.0 * delta;

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

    fn collide(
        &mut self,
        delta: f32,
        loader: &loader::ChunkLoader,
        (dx, dy, dz): (f32, f32, f32),
    ) -> (f32, f32, f32) {
        let (mut dx, mut dy, mut dz) = (dx, dy, dz);
        let (nx, ny, nz) = (self.x + dx, self.y + dy, self.z + dz);

        let player_box_current =
            create_player_aabb((self.x, self.y, self.z), (self.x, self.y, self.z));
        let player_box_stepped = create_player_aabb(
            (self.x.min(nx), self.y.min(ny), self.z.min(nz)),
            (self.x.max(nx), self.y.max(ny), self.z.max(nz)),
        );

        for x in (nx.floor() as i32 - 1)..(nx.floor() as i32 + 2) {
            for y in (ny.floor() as i32 - 1)..(ny.floor() as i32 + 3) {
                for z in (nz.floor() as i32 - 1)..(nz.floor() as i32 + 2) {
                    match loader.get_block([x, y, z]) {
                        None => {
                            self.velocity.1 = 0.0;
                            dy = 0.0;
                        }
                        Some(block) if !block.is_air() => {
                            let block_aabb = AABB::new(
                                Point3::new(x as f32, y as f32, z as f32),
                                Point3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                            );
                            if player_box_stepped.intersects(&block_aabb)
                                && !player_box_current.intersects(&block_aabb)
                            {
                                let x_box = create_player_aabb(
                                    (self.x.min(nx), self.y, self.z),
                                    (self.x.max(nx), self.y, self.z),
                                );
                                let y_box = create_player_aabb(
                                    (self.x, self.y.min(ny), self.z),
                                    (self.x, self.y.max(ny), self.z),
                                );
                                let z_box = create_player_aabb(
                                    (self.x, self.y, self.z.min(nz)),
                                    (self.x, self.y, self.z.max(nz)),
                                );

                                if x_box.intersects(&block_aabb) {
                                    self.velocity.0 = 0.0;
                                    dx = 0.0;
                                }

                                if y_box.intersects(&block_aabb) {
                                    self.velocity.1 = 0.0;
                                    dy = 0.0;
                                    self.falling = false;
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
            lin_speed: 6.0,
            rot_speed: 0.75,
            jump_power: 8.0,
            falling: true,
            camera: camera::Camera {
                x: 0.0,
                y: 100.0,
                z: 0.0,
                pitch: std::f32::consts::PI / 2.0,
                yaw: 0.0,
                roll: 0.0,
                projection: [[0.0; 4]; 4],
            },
            miner_builder: MinerBuilder::default(),
        }
    }
}

const HALF_WIDTH: f32 = 0.25;
const HEIGHT: f32 = 1.5;
const HALF_DEPTH: f32 = 0.25;

#[inline]
fn create_player_aabb(
    (x_min, y_min, z_min): (f32, f32, f32),
    (x_max, y_max, z_max): (f32, f32, f32),
) -> AABB {
    AABB::new(
        Point3::new(x_min - HALF_WIDTH, y_min, z_min - HALF_DEPTH),
        Point3::new(x_max + HALF_WIDTH, y_max + HEIGHT, z_max + HALF_DEPTH),
    )
}

fn cast_ray(start_point: [f32;3], rho: f32, phi: f32, theta: f32, loader: &ChunkLoader) -> [i32;3] {
    let ((sin_p, cos_p), (sin_t, cos_t)) = (phi.sin_cos(), theta.sin_cos());
    let ray_size = [rho * sin_p * cos_t,
                             rho * cos_p,
                             rho * sin_p * sin_t];

    let end_point = (start_point[0] + ray_size[0], start_point[1] + ray_size[1], start_point[2] + ray_size[2]);

    for (x, y, z) in line_drawing::WalkVoxels::new((start_point[0], start_point[1], start_point[2]), end_point, &line_drawing::VoxelOrigin::Corner) {
        if let Some(block) = loader.get_block([x,y,z]) {
            if !block.is_air() {
                return [x,y,z];
            }
        }
    }
    [start_point[0].floor() as i32, start_point[1].floor() as i32, start_point[2].floor() as i32]
}

/// Casts a ray and returns block coordinate of the air block in front of the block the ray hit, and None otherwise
fn cast_ray_in_front(start_point: [f32;3], rho: f32, phi: f32, theta: f32, loader: &ChunkLoader) -> Option<[i32;3] >{
    let ((sin_p, cos_p), (sin_t, cos_t)) = (phi.sin_cos(), theta.sin_cos());
    let ray_size = [rho * sin_p * cos_t,
                             rho * cos_p,
                             rho * sin_p * sin_t];

    let end_point = (start_point[0] + ray_size[0], start_point[1] + ray_size[1], start_point[2] + ray_size[2]);
    let mut last = [start_point[0].floor() as i32, start_point[1].floor() as i32, start_point[2].floor() as i32];
    for (x, y, z) in line_drawing::WalkVoxels::new((start_point[0], start_point[1], start_point[2]), end_point, &line_drawing::VoxelOrigin::Corner) {
        if let Some(block) = loader.get_block([x,y,z]) {
            if block.id() != 0 {
                return Some(last);
            }
        }
        last = [x,y,z];
    }
    None
}

#[inline]
fn mine(miner: &mut MinerBuilder, coord: [i32;3], delta: f32, speed: f32, loader: &mut ChunkLoader) {
    if miner.coord != coord {
        miner.reset_miner(coord);
    }
    miner.coord = coord;
    miner.update();
    let block = loader.get_block(coord).unwrap_or(Block::air());
    let health = block.health();
    miner.mining_progress += delta * speed;
    if health - miner.mining_progress <= 0.0 && !block.is_air() {
        loader.set_block(coord.clone(), Block::air());
        
    }
}

struct MinerBuilder {
    pub mining_progress: f32,
    coord: [i32;3],
    last_mine_time: Instant,
    last_build_time: Instant,
}

impl Default for MinerBuilder {
    fn default() -> Self {
        Self {
            mining_progress: Default::default(),
            coord: Default::default(),
            last_mine_time: Instant::now(),
            last_build_time: Instant::now()
        }
    }
}

impl MinerBuilder {
    pub fn reset_miner(&mut self, coord: [i32;3]) {
        self.mining_progress = 0.0;
        self.coord = coord;
    } 

    pub fn can_build(&mut self) -> bool {
        let now = Instant::now();
        if (now - self.last_build_time).as_millis() > 200 {
            self.last_build_time = now;
            return true;
        }
        false
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        if (now - self.last_mine_time).as_millis() > 80 {
            self.mining_progress = 0.0;
        }
        self.last_mine_time = now;
    }
}