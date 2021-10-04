use crate::math::Vector2;
use core::ops;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl ops::Mul<u32> for Viewport {
    type Output = Viewport;

    fn mul(self, rhs: u32) -> Viewport {
        Viewport {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl ops::Div<u32> for Viewport {
    type Output = Viewport;

    fn div(self, rhs: u32) -> Viewport {
        Viewport {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl Into<Vector2> for Viewport {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self.width as f32,
            y: self.height as f32,
        }
    }
}
