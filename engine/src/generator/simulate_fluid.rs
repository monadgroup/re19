use super::prelude::*;
use crate::math::Vector3;
use crate::renderer::fluid_sim_renderer::FluidProperties;

pub static SIMULATE_FLUID_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Simulate Fluid",
    instantiate_generator: |context| Box::new(SimulateFluid::new()),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "vorticity strength",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density dissipation",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density buoyancy",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density weight",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "temperature dissipation",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "velocity dissipation",
                    value_type: PropertyType::Float,
                },
            ],
        },
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "input",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "radius",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "pos",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "temperature",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "velocity",
                    value_type: PropertyType::Vec3,
                },
            ],
        },
    ],
};

pub struct SimulateFluid {
    last_frame: u32,
}

impl SimulateFluid {
    fn new() -> Self {
        SimulateFluid { last_frame: 0 }
    }
}

impl Generator for SimulateFluid {
    fn update(
        &mut self,
        _io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        let radius: Vector3 = prop(properties, 1, 0);
        let props = FluidProperties {
            input_radius: Vector3 {
                x: radius.x.max(0.001),
                y: radius.y.max(0.001),
                z: radius.z.max(0.001),
            },
            input_pos: prop(properties, 1, 1),
            density_amount: prop(properties, 1, 2),
            temperature_amount: prop(properties, 1, 3),
            velocity_amount: prop(properties, 1, 4),

            vorticity_strength: prop(properties, 0, 0),
            density_dissipation: prop(properties, 0, 1),
            density_buoyancy: prop(properties, 0, 2),
            density_weight: prop(properties, 0, 3),
            temperature_dissipation: prop(properties, 0, 4),
            velocity_dissipation: prop(properties, 0, 5),
        };

        while self.last_frame / 2 < local_frame / 2 {
            renderers.fluid.run(context, props);
            self.last_frame += 1;
        }

        self.last_frame = local_frame;
    }
}
