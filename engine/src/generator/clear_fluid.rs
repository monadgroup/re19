use super::prelude::*;

pub static CLEAR_FLUID_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Clear Fluid",
    instantiate_generator: |context| Box::new(ClearFluid),
    groups: &[],
};

pub struct ClearFluid;

impl Generator for ClearFluid {
    fn update(
        &mut self,
        _io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        _local_frame: u32,
        _properties: &[&[ClipPropertyValue]],
    ) {
        renderers.fluid.clear(context);
    }
}
