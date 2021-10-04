use crate::camera::PerspectiveCamera;
use crate::math::Matrix4;
use crate::shader_view::ShaderView;

pub trait PerspectiveCameraController {
    fn perspective_camera(&self) -> &PerspectiveCamera;
}

pub trait TransformController {
    fn transform(&self) -> &Matrix4;
}

pub trait CloudController {
    fn dimensions(&self) -> (u32, u32, u32);
    fn buffer_resource(&self) -> &ShaderView;
}
