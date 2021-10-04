pub mod bloom_renderer;
pub mod chromab_renderer;
pub mod clouds_renderer;
pub mod common;
pub mod fluid_sim_renderer;
pub mod fxaa_renderer;
pub mod godray_renderer;
pub mod grading_renderer;
pub mod launch_scene_renderer;
pub mod rocket_scene_renderer;
pub mod shadow_map_renderer;
pub mod skybox_renderer;
pub mod hills_scene_renderer;

use crate::creation_context::CreationContext;

pub struct RendererCollection {
    pub blit: self::common::BlitRenderer,
    pub fxaa: self::fxaa_renderer::FxaaRenderer,
    pub grading: self::grading_renderer::GradingRenderer,
    pub bloom: self::bloom_renderer::BloomRenderer,
    pub chromab: self::chromab_renderer::ChromabRenderer,
    pub skybox: self::skybox_renderer::SkyboxRenderer,
    pub godray: self::godray_renderer::GodrayRenderer,
    pub shadow_map: self::shadow_map_renderer::ShadowMapRenderer,
    pub fluid: self::fluid_sim_renderer::FluidSimRenderer,
    pub launch_scene: self::launch_scene_renderer::LaunchScene,
    pub rocket_scene: self::rocket_scene_renderer::RocketScene,
    pub clouds: self::clouds_renderer::CloudsRenderer,
    pub hills_scene: self::hills_scene_renderer::HillsScene,
}

impl RendererCollection {
    pub fn new(context: &mut CreationContext) -> Self {
        RendererCollection {
            blit: self::common::BlitRenderer::new(context),
            fxaa: self::fxaa_renderer::FxaaRenderer::new(context),
            grading: self::grading_renderer::GradingRenderer::new(context),
            bloom: self::bloom_renderer::BloomRenderer::new(context),
            chromab: self::chromab_renderer::ChromabRenderer::new(context),
            skybox: self::skybox_renderer::SkyboxRenderer::new(context),
            godray: self::godray_renderer::GodrayRenderer::new(context),
            shadow_map: self::shadow_map_renderer::ShadowMapRenderer::new(context),
            fluid: self::fluid_sim_renderer::FluidSimRenderer::new(context),
            launch_scene: self::launch_scene_renderer::LaunchScene::new(context),
            rocket_scene: self::rocket_scene_renderer::RocketScene::new(context),
            clouds: self::clouds_renderer::CloudsRenderer::new(context),
            hills_scene: self::hills_scene_renderer::HillsScene::new(context),
        }
    }
}
