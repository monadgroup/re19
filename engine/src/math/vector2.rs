use super::Float;
use core::{iter, ops};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn with_x(self, x: f32) -> Self {
        Vector2 { x, ..self }
    }

    pub fn with_y(self, y: f32) -> Self {
        Vector2 { y, ..self }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn unit(self) -> Self {
        self / Vector2::from(self.length())
    }

    pub fn lerp(self, other: Vector2, t: f32) -> Vector2 {
        self + (other - self) * Vector2::from(t)
    }
}

define_vec!(Vector2 => (x, y));

impl iter::Sum for Vector2 {
    fn sum<I: iter::Iterator<Item = Vector2>>(iter: I) -> Vector2 {
        let mut vec = Vector2::default();
        for v in iter {
            vec += v;
        }
        vec
    }
}

impl ops::Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Vector2 { x, y }
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from((x, y): (f32, f32)) -> Self {
        Vector2 { x, y }
    }
}

impl From<f32> for Vector2 {
    fn from(val: f32) -> Self {
        Vector2 { x: val, y: val }
    }
}

impl Into<[f32; 2]> for Vector2 {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<(f32, f32)> for Vector2 {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}
