use super::{Float, Matrix4, Vector3, Vector4};
use core::ops;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn axis(axis: Vector3, angle: f32) -> Self {
        let axis_vec = axis * (angle / 2.).sin();
        Quaternion {
            x: axis_vec.x,
            y: axis_vec.y,
            z: axis_vec.z,
            w: (angle / 2.).cos(),
        }
    }

    pub fn euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        let cy = (roll / 2.).cos();
        let sy = (roll / 2.).sin();
        let cr = (pitch / 2.).cos();
        let sr = (pitch / 2.).sin();
        let cp = (yaw / 2.).cos();
        let sp = (yaw / 2.).sin();

        Quaternion {
            x: cy * sr * cp - sy * cr * sp,
            y: cy * cr * sp + sy * sr * cp,
            z: sy * cr * cp - cy * sr * sp,
            w: cy * cr * cp + sy * sr * sp,
        }
    }

    pub fn slerp(self, target: Quaternion, amount: f32) -> Self {
        let lhs_vec: Vector4 = self.into();
        let rhs_vec: Vector4 = target.into();

        let dot = lhs_vec.dot(rhs_vec);
        if dot.abs() > 0.9995 {
            return lhs_vec.lerp(rhs_vec, amount).unit().into();
        }

        let (normal_rhs, normal_dot) = if dot < 0. {
            (-rhs_vec, -dot)
        } else {
            (rhs_vec, dot)
        };

        let clamped_dot = normal_dot.max(-1.).min(1.);
        let theta_0 = clamped_dot.acos();
        let theta = theta_0 * amount;

        let q2 = (normal_rhs - lhs_vec * clamped_dot).unit();
        (lhs_vec * theta.cos() + q2 * theta.sin()).into()
    }

    pub fn as_matrix(self) -> Matrix4 {
        let n = 2. / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);

        return Matrix4 {
            m: [
                [
                    1. - n * self.y * self.y - n * self.z * self.z,
                    n * self.x * self.y - n * self.z * self.w,
                    n * self.x * self.z + n * self.y * self.w,
                    0.,
                ],
                [
                    n * self.x * self.y + n * self.z * self.w,
                    1. - n * self.x * self.x - n * self.z * self.z,
                    n * self.y * self.z - n * self.x * self.w,
                    0.,
                ],
                [
                    n * self.x * self.z - n * self.y * self.w,
                    n * self.y * self.z + n * self.x * self.w,
                    1. - n * self.x * self.x - n * self.y * self.y,
                    0.,
                ],
                [0., 0., 0., 1.],
            ],
        };
    }

    pub fn as_euler(self) -> (f32, f32, f32) {
        let t0 = 2. * (self.w * self.x + self.y * self.z);
        let t1 = 1. - 2. * (self.x * self.x + self.y * self.y);
        let roll = t0.atan2(t1);

        let t2 = (2. * (self.w * self.y - self.z * self.x)).max(-1.).min(1.);
        let pitch = t2.asin();

        let t3 = 2. * (self.w * self.z + self.x * self.y);
        let t4 = 1. - 2. * (self.y * self.y + self.z * self.z);
        let yaw = t3.atan2(t4);

        (pitch, roll, yaw)
    }

    pub fn as_vector(self) -> Vector3 {
        Vector3 {
            x: 0.,
            y: 1.,
            z: 0.,
        } * self
    }

    pub fn as_forward(self) -> Vector3 {
        Vector3::unit_x() * self
    }

    pub fn as_up(self) -> Vector3 {
        Vector3::unit_y() * self
    }

    pub fn as_right(self) -> Vector3 {
        Vector3::unit_z() * self
    }

    pub fn normalize(self) -> Quaternion {
        let len = (self.x + self.y + self.z + self.w).sqrt();
        Quaternion {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w / len,
        }
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Quaternion {
            x: 0.,
            y: 0.,
            z: 0.,
            w: 1.,
        }
    }
}

impl ops::MulAssign<Quaternion> for Quaternion {
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs;
    }
}

impl ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            x: self.x * rhs.w + self.y * rhs.z - self.z * rhs.y + self.w * rhs.x,
            y: -self.x * rhs.z + self.y * rhs.w + self.z * rhs.x + self.w * rhs.y,
            z: self.x * rhs.y - self.y * rhs.x + self.z * rhs.w + self.w * rhs.z,
            w: -self.x * rhs.x - self.y * rhs.y - self.z * rhs.z + self.w * rhs.w,
        }
    }
}

impl ops::MulAssign<Quaternion> for Vector3 {
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs;
    }
}

impl ops::Mul<Quaternion> for Vector3 {
    type Output = Vector3;

    fn mul(self, q: Quaternion) -> Vector3 {
        let u = Vector3 {
            x: q.x,
            y: q.y,
            z: q.z,
        };
        let s = q.w;
        2. * u.dot(self) * u + (s * s - u.dot(u)) * self + 2. * s * u.cross(self)
    }
}

impl From<Vector4> for Quaternion {
    fn from(vec: Vector4) -> Quaternion {
        Quaternion {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: vec.w,
        }
    }
}

impl Into<Vector4> for Quaternion {
    fn into(self) -> Vector4 {
        Vector4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
