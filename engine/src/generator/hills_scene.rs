use super::prelude::*;
use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::camera::CameraBuffer;
use crate::math::{Matrix4, Quaternion, RgbColor, RgbaColor, Vector2, Vector3, Vector4};
use crate::mesh::{primitives, Mesh};
use crate::object::MeshObject;
use crate::raster_state::RasterState;
use crate::renderer::clouds_renderer::CloudsData;
use crate::renderer::common::{GaussBlurRenderer, PostRenderer, StandardRenderer};
use crate::resources::shader_manager::ComputeKey;
use crate::shader_view::ShaderView;
use crate::texture::{
    AddressMode, DepthStencil, PingPong2D, RenderTarget2D, Sampler, ShaderResource2D, Texture2D,
    Texture3D,
};
use crate::unordered_view::UnorderedView;
use crate::viewport::Viewport;
use core::{f32, ptr};
use winapi::shared::dxgiformat::{DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32_FLOAT};
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS,
    D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_VIEWPORT,
};

pub static HILLS_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Hills Scene",
    instantiate_generator: |context| Box::new(HillsScene),
    groups: &[
        SchemaGroup {
            #[cfg(debug_assertions)]
            name: "terrain",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "center",
                    value_type: PropertyType::Vec3,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "size",
                    value_type: PropertyType::Vec3,
                },
            ],
        },
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
            name: "fog",
            properties: &[
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "color",
                    value_type: PropertyType::RgbaColor,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "exp",
                    value_type: PropertyType::Float,
                },
                SchemaProperty {
                    #[cfg(debug_assertions)]
                    name: "density",
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
            ],
        },
    ],
};

pub struct HillsScene;

impl Generator for HillsScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        _local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        renderers.hills_scene.update(
            io,
            context,
            &mut renderers.clouds,
            &mut renderers.shadow_map,
            &mut renderers.godray,
            properties
        );
    }
}
