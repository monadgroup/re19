use crate::math::{Float, Vector2};

static C0: Vector2 = Vector2 { x: 0., y: 0. };
static C3: Vector2 = Vector2 { x: 1., y: 1. };

/// A cubic bezier curve for animation. For simplicity, it assumes that c0 is at (0, 0) and
/// c3 is at (1, 1).
#[derive(Debug, Clone)]
pub struct CubicBezier {
    c1: Vector2,
    c2: Vector2,

    p: [Vector2; 4],
}

impl CubicBezier {
    pub fn new(c1: Vector2, c2: Vector2) -> CubicBezier {
        CubicBezier {
            c1,
            c2,
            p: CubicBezier::calculate_polynomial(c1, c2),
        }
    }

    pub fn c1(&self) -> Vector2 {
        self.c1
    }

    pub fn set_c1(&mut self, c1: Vector2) {
        if c1 != self.c1 {
            self.c1 = c1;
            self.update_polynomial();
        }
    }

    pub fn c2(&self) -> Vector2 {
        self.c2
    }

    pub fn set_c2(&mut self, c2: Vector2) {
        if c2 != self.c2 {
            self.c2 = c2;
            self.update_polynomial();
        }
    }

    pub fn get_pos_at(&self, t: f32) -> Vector2 {
        ((self.p[0] * t + self.p[1]) * t + self.p[2]) * t + self.p[3]
    }

    pub fn get_y_at(&self, x: f32) -> f32 {
        let mut l = 0.;
        let mut u = 1.;
        let mut s = 0.5;
        let mut a = self.get_pos_at(s).x;

        while (x - a).abs() > 0.0001 {
            if x > a {
                l = s;
            } else {
                u = s;
            }

            s = (u + l) * 0.5;
            a = self.get_pos_at(s).x;
        }

        self.get_pos_at(s).y
    }

    fn update_polynomial(&mut self) {
        self.p = CubicBezier::calculate_polynomial(self.c1, self.c2);
    }

    fn calculate_polynomial(c1: Vector2, c2: Vector2) -> [Vector2; 4] {
        [
            C0 + 3. * (c1 - c2) + C3,
            3. * (C0 - 2. * c1 + c2),
            3. * (-C0 + c1),
            C0,
        ]
    }
}
