mod mesh_object;
mod quad_object;

pub use self::mesh_object::MeshObject;
pub use self::quad_object::QuadObject;

use crate::math::Matrix4;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ObjectBuffer {
    pub model_matrix: Matrix4,
    pub norm_model_matrix: Matrix4,
}

impl ObjectBuffer {
    pub fn new(model: Matrix4) -> Self {
        ObjectBuffer {
            model_matrix: model,
            norm_model_matrix: model.transform_normal(),
        }
    }
}
