use super::{Matrix4, Quaternion, Vector3};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Ray {
    pub pos: Vector3,
    pub dir: Quaternion,
}

impl Ray {
    pub fn as_matrix(&self) -> Matrix4 {
        Matrix4::translate(self.pos) * self.dir.as_matrix()
    }
}
