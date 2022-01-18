use crate::chunk::CHUNK_SIZE;

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl Camera {
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let direction = &[
            self.pitch.sin() * self.yaw.cos(),
            self.pitch.cos(),
            self.pitch.sin() * self.yaw.sin(),
        ];
        const UP: &[f32;3] = &[0.0, 1.0, 0.0];
        let f = {
            let f = direction;
            let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
            let len = len.sqrt();
            [f[0] / len, f[1] / len, f[2] / len]
        };

        let s = [
            UP[1] * f[2] - UP[2] * f[1],
            UP[2] * f[0] - UP[0] * f[2],
            UP[0] * f[1] - UP[1] * f[0],
        ];

        let s_norm = {
            let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
            let len = len.sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };

        let u = [
            f[1] * s_norm[2] - f[2] * s_norm[1],
            f[2] * s_norm[0] - f[0] * s_norm[2],
            f[0] * s_norm[1] - f[1] * s_norm[0],
        ];

        let p = [
            -self.x * s_norm[0] - self.y * s_norm[1] - self.z * s_norm[2],
            -self.x * u[0] - self.y * u[1] - self.z * u[2],
            -self.x * f[0] - self.y * f[1] - self.z * f[2],
        ];

        [
            [s_norm[0], u[0], f[0], 0.0],
            [s_norm[1], u[1], f[1], 0.0],
            [s_norm[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }

    pub fn perspective(&self, target: &impl glium::Surface) -> [[f32; 4]; 4] {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let fov: f32 = 3.141592 / 3.0;
        let zfar = 1024.0;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f * aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
            [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
        ]
    }
}

struct Plane {
    a: f32,
    b: f32,
    c: f32,
    d: f32
}

impl Plane {
    fn new(a: f32, b:f32, c: f32, d: f32) -> Self {
        let x = nalgebra::Vector4::new(a, b, c, d).normalize();
        Plane {
            a: x.x,
            b: x.y,
            c: x.z,
            d: x.w,
        }
    }
}

pub struct Frustum {
    planes: [Plane;6],
}

impl Frustum {
    pub fn new(vp: &[[f32;4];4]) -> Self {
        // Left
        let p1 = Plane::new(
            vp[0][3] + vp[0][0],
            vp[1][3] + vp[1][0],
            vp[2][3] + vp[2][0],
            vp[3][3] + vp[3][0],
        );

        // Right
        let p2 = Plane::new (
            vp[0][3] - vp[0][0],
            vp[1][3] - vp[1][0],
            vp[2][3] - vp[2][0],
            vp[3][3] - vp[3][0],
        );

        // Bottom
        let p3 = Plane::new (
            vp[0][3] + vp[0][1],
            vp[1][3] + vp[1][1],
            vp[2][3] + vp[2][1],
            vp[3][3] + vp[3][1],
        );

        // Top
        let p4 = Plane::new (
            vp[0][3] - vp[0][1],
            vp[1][3] - vp[1][1],
            vp[2][3] - vp[2][1],
            vp[3][3] - vp[3][1],
        );

        // Near
        let p5 = Plane::new (
            vp[0][3] + vp[0][2],
            vp[1][3] + vp[1][2],
            vp[2][3] + vp[2][2],
            vp[3][3] + vp[3][2],
        );

        // Far
        let p6 = Plane {
            a: vp[0][3] - vp[0][2],
            b: vp[1][3] - vp[1][2],
            c: vp[2][3] - vp[2][2],
            d: vp[3][3] - vp[3][2],
        };

        Frustum {
            planes: [p1, p2, p3, p4, p5, p6],
        }
    }

    pub fn contains(&self, chunk_coord: &[i32;3]) -> bool {
        let (xs, ys, zs) = ((chunk_coord[0] * CHUNK_SIZE.0 as i32) as f32, (chunk_coord[1] as i32 * CHUNK_SIZE.1 as i32) as f32, (chunk_coord[2] as i32 * CHUNK_SIZE.2 as i32) as f32);
        let (xf, yf, zf) = (xs + CHUNK_SIZE.0 as f32, ys + CHUNK_SIZE.1 as f32, zs + CHUNK_SIZE.2 as f32);
        for plane in &self.planes {
            if plane.a * xs + plane.b * ys + plane.c * zs + plane.d > 0.0 {
                continue;
            }
            if plane.a * xf + plane.b * ys + plane.c * zs + plane.d > 0.0 {
                continue;
            }
            if plane.a * xs + plane.b * yf + plane.c * zs + plane.d > 0.0 {
                continue;
            }
            if plane.a * xf + plane.b * yf + plane.c * zs + plane.d > 0.0 {
                continue;
            }
            if plane.a * xs + plane.b * ys + plane.c * zf + plane.d > 0.0 {
                continue; 
            }
            if plane.a * xf + plane.b * ys + plane.c * zf + plane.d > 0.0 {
               continue; 
            }
            if plane.a * xs + plane.b * yf + plane.c * zf + plane.d > 0.0 {
               continue; 
            }
            if plane.a * xf + plane.b * yf + plane.c * zf + plane.d > 0.0 {
                continue;
            }
            return false;
        }
        true
    }
}