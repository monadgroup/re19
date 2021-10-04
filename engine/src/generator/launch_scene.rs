use super::prelude::*;
use crate::renderer::launch_scene_renderer::CloudState;

pub static LAUNCH_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Launch Scene",
    instantiate_generator: |context| Box::new(LaunchScene::new(context)),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "light map",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "position",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "x range",
                    value_type: PropertyType::Vec2,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "y range",
                    value_type: PropertyType::Vec2,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "z range",
                    value_type: PropertyType::Vec2,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "rays",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "steps",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "step length",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "start dist",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "fluid",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "box pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "box size",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "march step length",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density multiplier",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "fluid shadow",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "directional <-> point",
                    value_type: PropertyType::Float,
                },
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
                    name: "size (directional)",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "radius (point)",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "max radius (point)",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "rocket",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "height",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "enabled",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "clouds",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "enabled",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "y",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "height",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "map offset",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "color",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "scatter color",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "light direction",
                    value_type: PropertyType::Rotation,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "opacity",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "gen once <-> live",
                    value_type: PropertyType::Float,
                },
            ],
        },
    ],
};

pub struct LaunchScene {
    cloud_state: CloudState,
}

impl LaunchScene {
    pub fn new(context: &mut CreationContext) -> Self {
        LaunchScene {
            cloud_state: CloudState::new(context),
        }
    }
}

impl Generator for LaunchScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        renderers.launch_scene.update(
            io,
            context,
            &mut renderers.shadow_map,
            &mut renderers.fluid,
            &mut renderers.godray,
            &mut renderers.clouds,
            &mut renderers.blit,
            &mut self.cloud_state,
            properties,
        );
    }
}
