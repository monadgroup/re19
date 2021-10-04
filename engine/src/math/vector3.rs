use super::{Float, Quaternion, Vector4};
use core::{f32, iter, ops};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn with_x(self, x: f32) -> Self {
        Vector3 { x, ..self }
    }

    pub fn with_y(self, y: f32) -> Self {
        Vector3 { y, ..self }
    }

    pub fn with_z(self, z: f32) -> Self {
        Vector3 { z, ..self }
    }

    pub fn dot(self, other: Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn unit(self) -> Vector3 {
        self / Vector3::from(self.length())
    }

    pub fn lerp(self, other: Vector3, t: f32) -> Vector3 {
        self + (other - self) * Vector3::from(t)
    }

    pub fn project(self) -> Vector4 {
        Vector4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: 1.,
        }
    }

    pub fn as_vec4(self, w: f32) -> Vector4 {
        Vector4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w,
        }
    }

    pub fn floor(self) -> Vector3 {
        Vector3 {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }

    pub fn fract(self) -> Vector3 {
        Vector3 {
            x: self.x.fract(),
            y: self.y.fract(),
            z: self.z.fract(),
        }
    }

    pub fn get_rotation_to(self, dest: Vector3, fallback_axis: Vector3) -> Quaternion {
        let v0 = self.unit();
        let v1 = dest.unit();
        let d = v0.dot(v1);
        if d >= 1. {
            Quaternion::default()
        } else if d < (1e-6 - 1.) {
            Quaternion::axis(fallback_axis, f32::consts::PI)
        } else {
            let s = ((1. + d) * 2.).sqrt();
            let inv_s = 1. / s;
            let c = v0.cross(v1);

            Quaternion {
                x: c.x * inv_s,
                y: c.y * inv_s,
                z: c.z * inv_s,
                w: s * 0.5,
            }
            .normalize()
        }
    }

    pub fn is_close_to(self, b: Vector3) -> bool {
        let delta = self - b;
        delta.x.abs() <= f32::EPSILON
            && delta.y.abs() <= f32::EPSILON
            && delta.z.abs() <= f32::EPSILON
    }

    pub fn unit_x() -> Self {
        Vector3 {
            x: 1.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn unit_y() -> Self {
        Vector3 {
            x: 0.,
            y: 1.,
            z: 0.,
        }
    }

    pub fn unit_z() -> Self {
        Vector3 {
            x: 0.,
            y: 0.,
            z: 1.,
        }
    }
}

define_vec!(Vector3 => (x, y, z));

impl iter::Sum for Vector3 {
    fn sum<I: iter::Iterator<Item = Vector3>>(iter: I) -> Vector3 {
        let mut vec = Vector3::default();
        for v in iter {
            vec += v;
        }
        vec
    }
}

impl ops::Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Vector3 { x, y, z }
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Vector3 { x, y, z }
    }
}

impl From<f32> for Vector3 {
    fn from(val: f32) -> Self {
        Vector3 {
            x: val,
            y: val,
            z: val,
        }
    }
}

impl Into<[f32; 3]> for Vector3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Into<(f32, f32, f32)> for Vector3 {
    fn into(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
