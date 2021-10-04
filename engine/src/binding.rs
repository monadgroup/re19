#[derive(Clone, Copy)]
pub struct PropertyBinding {
    pub group: usize,
    pub prop: usize,
}

impl PropertyBinding {
    pub fn new(group: usize, prop: usize) -> Self {
        PropertyBinding { group, prop }
    }
}

pub trait CameraBinding {
    fn camera_position_binding(&self) -> PropertyBinding;
    fn camera_direction_binding(&self) -> PropertyBinding;
    fn camera_fov_binding(&self) -> PropertyBinding;
}
