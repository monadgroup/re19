use super::{Float, Vector3};
use core::ops;

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn with_x(self, x: f32) -> Self {
        Vector4 { x, ..self }
    }

    pub fn with_y(self, y: f32) -> Self {
        Vector4 { y, ..self }
    }

    pub fn with_z(self, z: f32) -> Self {
        Vector4 { z, ..self }
    }

    pub fn with_w(self, w: f32) -> Self {
        Vector4 { w, ..self }
    }

    pub fn dot(self, other: Vector4) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn unit(self) -> Vector4 {
        self / Vector4::from(self.length())
    }

    pub fn lerp(self, other: Vector4, t: f32) -> Vector4 {
        self + (other - self) * t
    }

    pub fn unproject(self) -> Vector3 {
        Vector3 {
            x: self.x / self.w,
            y: self.y / self.w,
            z: self.z / self.w,
        }
    }

    pub fn as_vec3(self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn floor(self) -> Vector4 {
        Vector4 {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
            w: self.w.floor(),
        }
    }

    pub fn fract(self) -> Vector4 {
        Vector4 {
            x: self.x.fract(),
            y: self.y.fract(),
            z: self.z.fract(),
            w: self.w.fract(),
        }
    }
}

define_vec!(Vector4 => (x, y, z, w));

impl ops::Neg for Vector4 {
    type Output = Vector4;

    fn neg(self) -> Vector4 {
        Vector4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl From<[f32; 4]> for Vector4 {
    fn from([x, y, z, w]: [f32; 4]) -> Self {
        Vector4 { x, y, z, w }
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self {
        Vector4 { x, y, z, w }
    }
}

impl From<f32> for Vector4 {
    fn from(val: f32) -> Self {
        Vector4 {
            x: val,
            y: val,
            z: val,
            w: val,
        }
    }
}

impl Into<[f32; 4]> for Vector4 {
    fn into(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl Into<(f32, f32, f32, f32)> for Vector4 {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.z, self.w)
    }
}
