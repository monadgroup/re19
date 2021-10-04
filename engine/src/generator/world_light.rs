use super::prelude::*;
use crate::frame_context::LightBuffer;
use crate::math::{Quaternion, RgbaColor, Vector3, Vector4};

pub static WORLD_LIGHT_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "World Light",
    instantiate_generator: |_| Box::new(WorldLight),
    groups: &[SchemaGroup {
        #[cfg(debug_assertions)]
        name: "",
        properties: &[
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "direction",
                value_type: PropertyType::Rotation,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "color",
                value_type: PropertyType::RgbaColor,
            },
            SchemaProperty {
                #[cfg(debug_assertions)]
                name: "ambient",
                value_type: PropertyType::Float,
            },
        ],
    }],
};

pub struct WorldLight;

impl Generator for WorldLight {
    fn update(
        &mut self,
        _io: &mut GBuffer,
        context: &mut FrameContext,
        _renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        let light_buffer = LightBuffer {
            world_light_direction: prop::<Quaternion>(properties, 0, 0).as_right().as_vec4(0.),
            world_light_color: prop::<RgbaColor>(properties, 0, 1).premult().0.as_vec4(0.),
            world_light_ambient: prop(properties, 0, 2),
            world_light_rotation: prop::<Quaternion>(properties, 0, 0),
        };
        context
            .common
            .light_buffer
            .upload(context.devcon, light_buffer);
        context.common.light_data = light_buffer;
    }
}
