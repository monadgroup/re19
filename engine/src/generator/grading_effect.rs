use super::prelude::*;
use crate::renderer::bloom_renderer::{CompositeData, ExtractData};
use crate::renderer::grading_renderer::GradingParameters;

pub static GRADING_EFFECT_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Grading",
    instantiate_generator: |_| Box::new(GradingEffect),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "camera",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "exposure",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "vignette offset",
                    value_type: PropertyType::Vec2,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "vignette strength",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "vignette size",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "vignette power",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "chromab",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "grain",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "fade",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "bloom",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "shape",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "multiplier",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "bias",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "power",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "amount",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "size",
                    value_type: PropertyType::Vec2,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "bloom extract",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "extract multiplier",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "extract bias",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "extract power",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "extract amount",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "grading",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "curve",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gradient amt",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gradient start",
                    value_type: PropertyType::Vec2,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gradient a",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gradient end",
                    value_type: PropertyType::Vec2,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gradient b",
                    value_type: PropertyType::RgbaColor,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "tonemapping",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "a",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "b",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "c",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "d",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "e",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "f",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "w",
                    value_type: PropertyType::Float,
                },
            ],
        },
    ],
};

pub struct GradingEffect;

impl Generator for GradingEffect {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        // Render bloom
        let extract_data = ExtractData {
            multiplier: properties[2][0].value.into_float().unwrap(),
            bias: properties[2][1].value.into_float().unwrap(),
            power: properties[2][2].value.into_float().unwrap(),
            amount: properties[2][3].value.into_float().unwrap(),
        };
        let composite_data = CompositeData {
            shape: properties[1][0].value.into_float().unwrap(),
            multiplier: properties[1][1].value.into_float().unwrap(),
            bias: properties[1][2].value.into_float().unwrap(),
            power: properties[1][3].value.into_float().unwrap(),
            amount: properties[1][4].value.into_float().unwrap(),
        };
        io.swap_lit();
        renderers.bloom.render(
            context,
            io.read_output(),
            io.write_output(),
            properties[1][5].value.into_vec2().unwrap(),
            extract_data,
            composite_data,
        );

        // Render grading shader
        let grading_query = context.perf.start_gpu_str("grading");
        io.swap_lit();
        renderers.grading.render(
            context,
            GradingParameters {
                exposure: properties[0][0].value.into_float().unwrap(),
                fade: properties[0][7].value.into_float().unwrap(),
                curve: properties[3][0].value.into_vec3().unwrap(),
                vignette_offset: properties[0][1].value.into_vec2().unwrap(),
                vignette_strength: properties[0][2].value.into_float().unwrap(),
                vignette_size: properties[0][3].value.into_float().unwrap(),
                vignette_power: properties[0][4].value.into_float().unwrap(),

                gradient_dry_wet: properties[3][1].value.into_float().unwrap(),
                gradient_pos_a: properties[3][2].value.into_vec2().unwrap(),
                gradient_color_a: properties[3][3].value.into_rgba_color().unwrap().premult(),
                gradient_pos_b: properties[3][4].value.into_vec2().unwrap(),
                gradient_color_b: properties[3][5].value.into_rgba_color().unwrap().premult(),

                tonemap_a: properties[4][0].value.into_float().unwrap(),
                tonemap_b: properties[4][1].value.into_float().unwrap(),
                tonemap_c: properties[4][2].value.into_float().unwrap(),
                tonemap_d: properties[4][3].value.into_float().unwrap(),
                tonemap_e: properties[4][4].value.into_float().unwrap(),
                tonemap_f: properties[4][5].value.into_float().unwrap(),
                tonemap_w: properties[4][6].value.into_float().unwrap(),
            },
            io.read_output(),
            io.write_output(),
        );
        context.perf.end(grading_query);

        // Render FXAA shader
        let fxaa_query = context.perf.start_gpu_str("fxaa");
        io.swap_lit();
        renderers
            .fxaa
            .render(context, io.read_output(), io.write_output());
        context.perf.end(fxaa_query);

        // Render chromab
        let chromab_query = context.perf.start_gpu_str("chromab");
        io.swap_lit();
        renderers.chromab.render(
            context,
            properties[0][5].value.into_float().unwrap(),
            properties[0][6].value.into_float().unwrap(),
            io.read_output(),
            io.write_output(),
        );
        context.perf.end(chromab_query);
    }
}
