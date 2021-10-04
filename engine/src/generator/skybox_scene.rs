use super::prelude::*;

pub static SKYBOX_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Skybox",
    instantiate_generator: |_| Box::new(SkyboxScene),
    groups: &[],
};

pub struct SkyboxScene;

impl Generator for SkyboxScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        _local_frame: u32,
        _properties: &[&[ClipPropertyValue]],
    ) {
        io.depth_map().clear(context.devcon);
        renderers.skybox.render(context, io);
    }
}
