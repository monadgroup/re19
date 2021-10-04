use super::prelude::*;
use crate::buffer::{Buffer, InitialData};
use crate::math::{RgbColor, RgbaColor, Vector3};
use crate::renderer::common::PostRenderer;
use core::ptr;
use winapi::um::d3d11::D3D11_BIND_CONSTANT_BUFFER;

const GRADIENT_COUNT: usize = 9;

#[derive(Clone, Copy)]
#[repr(C)]
struct GradientData {
    rocket_pos: Vector3,
    rocket_size: f32,
    booster_separation: f32,
    brightness: f32,
    _pad0: u32,
    _pad1: u32,
    colors: [(RgbColor, u32); GRADIENT_COUNT + 1],
    height_curves: [(f32, f32, u32, u32); GRADIENT_COUNT],
}

pub static GRADIENT_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Gradient Scene",
    instantiate_generator: |context| Box::new(GradientScene::new(context)),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "brightness",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "rocket pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "rocket size",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "separation",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "heights",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h1",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h2",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h3",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h4",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h5",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h6",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h7",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h8",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "h9",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "curves",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c1",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c2",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c3",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c4",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c5",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c6",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c7",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c8",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c9",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "colors",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c1",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c2",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c3",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c4",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c5",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c6",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c7",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c8",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c9",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c10",
                    value_type: PropertyType::RgbaColor,
                },
            ],
        },
    ],
};

pub struct GradientScene {
    gradient_data: Buffer<GradientData>,
    renderer: PostRenderer,
}

impl GradientScene {
    pub fn new(context: &mut CreationContext) -> Self {
        GradientScene {
            gradient_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            renderer: PostRenderer::new(context, "gradient_scene.ps"),
        }
    }
}

impl Generator for GradientScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        _renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        {
            let mapped_data = self.gradient_data.map(context.devcon);
            let gradient_data = &mut mapped_data.slice_mut()[0];
            gradient_data.rocket_pos = prop(properties, 0, 1);
            gradient_data.rocket_size = prop(properties, 0, 2);
            gradient_data.booster_separation = prop(properties, 0, 3);
            gradient_data.brightness = prop(properties, 0, 0);

            let mut last_height = 0.;
            for height_index in 0..GRADIENT_COUNT {
                last_height += prop::<f32>(properties, 1, height_index);
                gradient_data.height_curves[height_index].0 = last_height;
                gradient_data.height_curves[height_index].1 = prop(properties, 2, height_index);
            }
            for color_index in 0..(GRADIENT_COUNT + 1) {
                gradient_data.colors[color_index].0 =
                    prop::<RgbaColor>(properties, 3, color_index).premult();
            }
        }

        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                2,
                &[context.common.camera_buffer.ptr(), self.gradient_data.ptr()][0],
            );
        }

        self.renderer.render(context, io.write_output(), true, true);

        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
        }
    }
}
