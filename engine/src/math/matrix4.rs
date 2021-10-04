use super::{Float, Vector2, Vector3, Vector4};
use core::{f32, ops};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4 {
    pub fn new(m: [[f32; 4]; 4]) -> Self {
        Matrix4 { m }
    }

    pub fn rotate_axis(axis: Vector3, angle: f32) -> Matrix4 {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();
        let inv_cos = 1. - cos_angle;

        Matrix4::new([
            [
                axis.x * axis.x * inv_cos + cos_angle,
                axis.y * axis.x * inv_cos - sin_angle * axis.z,
                axis.z * axis.x * inv_cos + sin_angle * axis.y,
                0.,
            ],
            [
                axis.x * axis.y * inv_cos + sin_angle * axis.z,
                axis.y * axis.y * inv_cos + cos_angle,
                axis.z * axis.y * inv_cos - sin_angle * axis.x,
                0.,
            ],
            [
                axis.x * axis.z * inv_cos - sin_angle * axis.y,
                axis.y * axis.z * inv_cos + sin_angle * axis.x,
                axis.z * axis.z * inv_cos + cos_angle,
                0.,
            ],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotate_x(angle: f32) -> Matrix4 {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        Matrix4::new([
            [1., 0., 0., 0.],
            [0., cos_angle, sin_angle, 0.],
            [0., -sin_angle, cos_angle, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotate_y(angle: f32) -> Matrix4 {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        Matrix4::new([
            [cos_angle, 0., -sin_angle, 0.],
            [0., 1., 0., 0.],
            [sin_angle, 0., cos_angle, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotate_z(angle: f32) -> Matrix4 {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        Matrix4::new([
            [cos_angle, -sin_angle, 0., 0.],
            [sin_angle, cos_angle, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn shear_x(y: f32, z: f32) -> Matrix4 {
        Matrix4::new([
            [1., 0., 0., 0.],
            [y, 1., 0., 0.],
            [z, 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn shear_y(x: f32, z: f32) -> Matrix4 {
        Matrix4::new([
            [1., x, 0., 0.],
            [0., 1., 0., 0.],
            [0., z, 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn shear_z(x: f32, y: f32) -> Matrix4 {
        Matrix4::new([
            [1., 0., x, 0.],
            [0., 1., y, 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn translate(vec: Vector3) -> Matrix4 {
        Matrix4::new([
            [1., 0., 0., vec.x],
            [0., 1., 0., vec.y],
            [0., 0., 1., vec.z],
            [0., 0., 0., 1.],
        ])
    }

    pub fn scale(vec: Vector3) -> Matrix4 {
        Matrix4::new([
            [vec.x, 0., 0., 0.],
            [0., vec.y, 0., 0.],
            [0., 0., vec.z, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn view(pos: Vector3, forward: Vector3, up: Vector3) -> Matrix4 {
        let right = forward.cross(up);
        Matrix4::new([
            [right.x, up.x, forward.x, pos.x],
            [right.y, up.y, forward.y, pos.y],
            [right.z, up.z, forward.z, pos.z],
            [0., 0., 0., 1.],
        ])
        .inverted()
    }

    pub fn project_perspective(fov_y: f32, aspect: f32, near_z: f32, far_z: f32) -> Matrix4 {
        let tan_v = fov_y.tan();
        let height = 1. / tan_v;
        let width = height / aspect;
        let f_range = far_z / (far_z - near_z);

        Matrix4::new([
            [width, 0., 0., 0.],
            [0., height, 0., 0.],
            [0., 0., f_range, -f_range * near_z],
            [0., 0., 1., 0.],
        ])
    }

    pub fn project_orthographic(x_clip: Vector2, y_clip: Vector2, z_clip: Vector2) -> Matrix4 {
        let x = 2. / (x_clip.y - x_clip.x);
        let y = 2. / (y_clip.y - y_clip.x);
        let z = -2. / (z_clip.y - z_clip.x);
        let c = -(x_clip.y + x_clip.x) / (x_clip.y - x_clip.x);
        let d = -(y_clip.y + y_clip.x) / (y_clip.y - y_clip.x);
        let e = -(z_clip.y + z_clip.x) / (z_clip.y - z_clip.x);

        Matrix4::new([
            [x, 0., 0., c],
            [0., y, 0., d],
            [0., 0., z, e],
            [0., 0., 0., 1.],
        ])
    }

    pub fn row(&self, row: usize) -> Vector4 {
        Vector4::from(self.m[row])
    }

    pub fn with_row(mut self, row: usize, vec: Vector4) -> Matrix4 {
        self.m[row] = vec.into();
        self
    }

    pub fn column(&self, column: usize) -> Vector4 {
        Vector4 {
            x: self.m[0][column],
            y: self.m[1][column],
            z: self.m[2][column],
            w: self.m[3][column],
        }
    }

    pub fn with_column(mut self, column: usize, vec: Vector4) -> Matrix4 {
        self.m[0][column] = vec.x;
        self.m[1][column] = vec.y;
        self.m[2][column] = vec.z;
        self.m[3][column] = vec.w;
        self
    }

    pub fn diagonal(&self) -> Vector4 {
        Vector4 {
            x: self.m[0][0],
            y: self.m[1][1],
            z: self.m[2][2],
            w: self.m[3][3],
        }
    }

    pub fn with_diagonal(mut self, vec: Vector4) -> Matrix4 {
        self.m[0][0] = vec.x;
        self.m[1][1] = vec.y;
        self.m[2][2] = vec.z;
        self.m[3][3] = vec.w;
        self
    }

    pub fn transposed(self) -> Matrix4 {
        Matrix4::new([
            [self.m[0][0], self.m[1][0], self.m[2][0], self.m[3][0]],
            [self.m[0][1], self.m[1][1], self.m[2][1], self.m[3][1]],
            [self.m[0][2], self.m[1][2], self.m[2][2], self.m[3][2]],
            [self.m[0][3], self.m[1][3], self.m[2][3], self.m[3][3]],
        ])
    }

    pub fn inverted(self) -> Matrix4 {
        let inv = Matrix4::new([
            [
                self[5] * self[10] * self[15]
                    - self[5] * self[11] * self[14]
                    - self[9] * self[6] * self[15]
                    + self[9] * self[7] * self[14]
                    + self[13] * self[6] * self[11]
                    - self[13] * self[7] * self[10],
                -self[1] * self[10] * self[15]
                    + self[1] * self[11] * self[14]
                    + self[9] * self[2] * self[15]
                    - self[9] * self[3] * self[14]
                    - self[13] * self[2] * self[11]
                    + self[13] * self[3] * self[10],
                self[1] * self[6] * self[15]
                    - self[1] * self[7] * self[14]
                    - self[5] * self[2] * self[15]
                    + self[5] * self[3] * self[14]
                    + self[13] * self[2] * self[7]
                    - self[13] * self[3] * self[6],
                -self[1] * self[6] * self[11]
                    + self[1] * self[7] * self[10]
                    + self[5] * self[2] * self[11]
                    - self[5] * self[3] * self[10]
                    - self[9] * self[2] * self[7]
                    + self[9] * self[3] * self[6],
            ],
            [
                -self[4] * self[10] * self[15]
                    + self[4] * self[11] * self[14]
                    + self[8] * self[6] * self[15]
                    - self[8] * self[7] * self[14]
                    - self[12] * self[6] * self[11]
                    + self[12] * self[7] * self[10],
                self[0] * self[10] * self[15]
                    - self[0] * self[11] * self[14]
                    - self[8] * self[2] * self[15]
                    + self[8] * self[3] * self[14]
                    + self[12] * self[2] * self[11]
                    - self[12] * self[3] * self[10],
                -self[0] * self[6] * self[15]
                    + self[0] * self[7] * self[14]
                    + self[4] * self[2] * self[15]
                    - self[4] * self[3] * self[14]
                    - self[12] * self[2] * self[7]
                    + self[12] * self[3] * self[6],
                self[0] * self[6] * self[11]
                    - self[0] * self[7] * self[10]
                    - self[4] * self[2] * self[11]
                    + self[4] * self[3] * self[10]
                    + self[8] * self[2] * self[7]
                    - self[8] * self[3] * self[6],
            ],
            [
                self[4] * self[9] * self[15]
                    - self[4] * self[11] * self[13]
                    - self[8] * self[5] * self[15]
                    + self[8] * self[7] * self[13]
                    + self[12] * self[5] * self[11]
                    - self[12] * self[7] * self[9],
                -self[0] * self[9] * self[15]
                    + self[0] * self[11] * self[13]
                    + self[8] * self[1] * self[15]
                    - self[8] * self[3] * self[13]
                    - self[12] * self[1] * self[11]
                    + self[12] * self[3] * self[9],
                self[0] * self[5] * self[15]
                    - self[0] * self[7] * self[13]
                    - self[4] * self[1] * self[15]
                    + self[4] * self[3] * self[13]
                    + self[12] * self[1] * self[7]
                    - self[12] * self[3] * self[5],
                -self[0] * self[5] * self[11]
                    + self[0] * self[7] * self[9]
                    + self[4] * self[1] * self[11]
                    - self[4] * self[3] * self[9]
                    - self[8] * self[1] * self[7]
                    + self[8] * self[3] * self[5],
            ],
            [
                -self[4] * self[9] * self[14]
                    + self[4] * self[10] * self[13]
                    + self[8] * self[5] * self[14]
                    - self[8] * self[6] * self[13]
                    - self[12] * self[5] * self[10]
                    + self[12] * self[6] * self[9],
                self[0] * self[9] * self[14]
                    - self[0] * self[10] * self[13]
                    - self[8] * self[1] * self[14]
                    + self[8] * self[2] * self[13]
                    + self[12] * self[1] * self[10]
                    - self[12] * self[2] * self[9],
                -self[0] * self[5] * self[14]
                    + self[0] * self[6] * self[13]
                    + self[4] * self[1] * self[14]
                    - self[4] * self[2] * self[13]
                    - self[12] * self[1] * self[6]
                    + self[12] * self[2] * self[5],
                self[0] * self[5] * self[10]
                    - self[0] * self[6] * self[9]
                    - self[4] * self[1] * self[10]
                    + self[4] * self[2] * self[9]
                    + self[8] * self[1] * self[6]
                    - self[8] * self[2] * self[5],
            ],
        ]);

        let inv_det = self[0] * inv[0] + self[1] * inv[4] + self[2] * inv[8] + self[3] * inv[12];
        //check_ne!(inv_det, 0.);
        inv / inv_det
    }

    pub fn transform_normal(self) -> Matrix4 {
        self.inverted().transposed()
    }

    pub fn mul_norm(self, rhs: Vector3) -> Vector3 {
        (self * rhs.as_vec4(0.)).as_vec3().unit()
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Matrix4::new([
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }
}

impl ops::Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &f32 {
        &self.m[index.0][index.1]
    }
}

impl ops::Index<usize> for Matrix4 {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        &self.m[index / 4][index % 4]
    }
}

fn multiply_row(row: usize, left: Matrix4, right: Matrix4) -> [f32; 4] {
    let left_row = left.row(row);

    [
        left_row.dot(right.column(0)),
        left_row.dot(right.column(1)),
        left_row.dot(right.column(2)),
        left_row.dot(right.column(3)),
    ]
}

impl ops::Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: Matrix4) -> Matrix4 {
        Matrix4::new([
            multiply_row(0, self, rhs),
            multiply_row(1, self, rhs),
            multiply_row(2, self, rhs),
            multiply_row(3, self, rhs),
        ])
    }
}

impl ops::MulAssign<Matrix4> for Matrix4 {
    fn mul_assign(&mut self, rhs: Matrix4) {
        *self = *self * rhs;
    }
}

impl ops::Mul<Vector4> for Matrix4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        Vector4 {
            x: rhs.dot(self.row(0)),
            y: rhs.dot(self.row(1)),
            z: rhs.dot(self.row(2)),
            w: rhs.dot(self.row(3)),
        }
    }
}

impl ops::Mul<Vector3> for Matrix4 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        (self * rhs.project()).unproject()
    }
}

impl ops::MulAssign<f32> for Matrix4 {
    fn mul_assign(&mut self, rhs: f32) {
        for row in self.m.iter_mut() {
            for cell in row.iter_mut() {
                *cell *= rhs;
            }
        }
    }
}

impl ops::DivAssign<f32> for Matrix4 {
    fn div_assign(&mut self, rhs: f32) {
        for row in self.m.iter_mut() {
            for cell in row.iter_mut() {
                *cell /= rhs;
            }
        }
    }
}

impl ops::Mul<f32> for Matrix4 {
    type Output = Matrix4;

    fn mul(mut self, rhs: f32) -> Matrix4 {
        self *= rhs;
        self
    }
}

impl ops::Div<f32> for Matrix4 {
    type Output = Matrix4;

    fn div(mut self, rhs: f32) -> Matrix4 {
        self /= rhs;
        self
    }
}

impl ops::AddAssign<Matrix4> for Matrix4 {
    fn add_assign(&mut self, rhs: Matrix4) {
        for (self_row, rhs_row) in self.m.iter_mut().zip(rhs.m.iter()) {
            for (self_cell, rhs_cell) in self_row.iter_mut().zip(rhs_row.iter()) {
                *self_cell += *rhs_cell;
            }
        }
    }
}

impl ops::SubAssign<Matrix4> for Matrix4 {
    fn sub_assign(&mut self, rhs: Matrix4) {
        for (self_row, rhs_row) in self.m.iter_mut().zip(rhs.m.iter()) {
            for (self_cell, rhs_cell) in self_row.iter_mut().zip(rhs_row.iter()) {
                *self_cell -= *rhs_cell;
            }
        }
    }
}

impl ops::Add<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn add(mut self, rhs: Matrix4) -> Matrix4 {
        self += rhs;
        self
    }
}

impl ops::Sub<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn sub(mut self, rhs: Matrix4) -> Matrix4 {
        self -= rhs;
        self
    }
}
