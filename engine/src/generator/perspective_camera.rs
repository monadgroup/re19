use super::prelude::*;
use crate::binding::{CameraBinding, PropertyBinding};
use crate::camera;
use crate::camera::Camera;
use crate::controller::PerspectiveCameraController;
use crate::math::Vector2;

pub static PERSPECTIVE_CAMERA_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Perspective Camera",
    instantiate_generator: |_| Box::new(PerspectiveCamera::new()),
    groups: &[SchemaGroup {
        #[cfg(debug_assertions)]
        name: "",
        properties: &[
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "base pos",
                value_type: PropertyType::Vec3,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "gymbal dir",
                value_type: PropertyType::Rotation,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "arm length",
                value_type: PropertyType::Float,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "head dir",
                value_type: PropertyType::Rotation,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "fov",
                value_type: PropertyType::Float,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "z range",
                value_type: PropertyType::Vec2,
            },
        ],
    }],
};

pub struct PerspectiveCamera {
    camera: camera::PerspectiveCamera,
}

impl PerspectiveCamera {
    pub fn new() -> Self {
        PerspectiveCamera {
            camera: camera::PerspectiveCamera::default(),
        }
    }
}

impl Generator for PerspectiveCamera {
    fn update(
        &mut self,
        _io: &mut GBuffer,
        context: &mut FrameContext,
        _renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        self.camera.base_pos = prop(properties, 0, 0);
        self.camera.gymbal_dir = prop(properties, 0, 1);
        self.camera.arm_length = prop(properties, 0, 2);
        self.camera.camera_dir = prop(properties, 0, 3);
        self.camera.fov = prop::<f32>(properties, 0, 4).to_radians();

        let z_range: Vector2 = prop(properties, 0, 5);
        self.camera.near_z = z_range.x;
        self.camera.far_z = z_range.y;

        self.camera.update(context.viewport);
        self.camera.upload(context.devcon, &mut context.common);
    }

    fn camera_binding(&self) -> Option<&dyn CameraBinding> {
        Some(self)
    }
}

impl CameraBinding for PerspectiveCamera {
    fn camera_position_binding(&self) -> PropertyBinding {
        PropertyBinding::new(0, 0)
    }

    fn camera_direction_binding(&self) -> PropertyBinding {
        PropertyBinding::new(0, 3)
    }

    fn camera_fov_binding(&self) -> PropertyBinding {
        PropertyBinding::new(0, 4)
    }
}
