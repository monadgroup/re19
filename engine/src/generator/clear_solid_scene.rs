use super::prelude::*;

pub static CLEAR_SOLID_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Clear Solid",
    instantiate_generator: |_| Box::new(ClearSolidScene),
    groups: &[SchemaGroup {
        #[cfg(debug_assertions)]
        name: "",
        properties: &[SchemaProperty {
            #[cfg(debug_assertions)]
            name: "color",
            value_type: PropertyType::RgbaColor,
        }],
    }],
};

pub struct ClearSolidScene;

impl Generator for ClearSolidScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        _renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        io.clear(context.devcon, prop(properties, 0, 0));
    }
}
