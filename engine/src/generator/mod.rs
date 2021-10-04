mod clear_fluid;
mod clear_solid_scene;
mod credits_scene;
mod gradient_scene;
mod grading_effect;
mod hills_scene;
mod launch_scene;
mod perspective_camera;
mod prelude;
mod rocket_scene;
mod simulate_fluid;
mod skybox_scene;
mod world_light;

use crate::animation::clip::ClipPropertyValue;
use crate::animation::schema::GeneratorSchema;
use crate::binding::CameraBinding;
use crate::controller::{CloudController, PerspectiveCameraController, TransformController};
use crate::frame_context::FrameContext;
use crate::gbuffer::GBuffer;
use crate::renderer::RendererCollection;

pub static GENERATOR_SCHEMAS: &[GeneratorSchema] = &[
    self::clear_solid_scene::CLEAR_SOLID_SCENE_SCHEMA,
    self::perspective_camera::PERSPECTIVE_CAMERA_SCHEMA,
    self::grading_effect::GRADING_EFFECT_SCHEMA,
    self::rocket_scene::ROCKET_SCENE_SCHEMA,
    self::launch_scene::LAUNCH_SCENE_SCHEMA,
    self::skybox_scene::SKYBOX_SCENE_SCHEMA,
    self::world_light::WORLD_LIGHT_SCHEMA,
    self::gradient_scene::GRADIENT_SCENE_SCHEMA,
    self::hills_scene::HILLS_SCENE_SCHEMA,
    self::simulate_fluid::SIMULATE_FLUID_SCHEMA,
    self::clear_fluid::CLEAR_FLUID_SCHEMA,
    self::credits_scene::CREDITS_SCENE_SCHEMA,
];

pub trait Generator: 'static {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    );

    // Binding access functions
    fn camera_binding(&self) -> Option<&dyn CameraBinding> {
        None
    }
}
