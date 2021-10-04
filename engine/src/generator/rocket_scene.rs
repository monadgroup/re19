use super::prelude::*;

pub static ROCKET_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Rocket Scene",
    instantiate_generator: |context| Box::new(RocketScene),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "density vol",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "size",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "light map blur",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "directional light",
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
                    name: "shadow vol pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "shadow vol size",
                    value_type: PropertyType::Vec3,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "point light",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "color",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "radius",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "max radius",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "ambient light",
            properties: &[SchemaProperty {
                #[cfg(debug_assertions)]
                name: "color",
                value_type: PropertyType::RgbaColor,
            }],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "rocket",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "base pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "enabled",
                    value_type: PropertyType::Float,
                }
            ],
        },
    ],
};

struct RocketScene;

impl Generator for RocketScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        renderers.rocket_scene.update(io, context, properties);
    }
}
