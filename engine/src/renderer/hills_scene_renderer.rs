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
use crate::renderer::clouds_renderer::CloudsRenderer;
use crate::renderer::shadow_map_renderer::ShadowMapRenderer;
use crate::renderer::godray_renderer::GodrayRenderer;
use crate::gbuffer::GBuffer;
use crate::frame_context::FrameContext;
use crate::creation_context::CreationContext;
use crate::animation::clip::ClipPropertyValue;
use crate::animation::property::prop;

#[derive(Clone, Copy)]
#[repr(C)]
struct TerrainData {
    pub fog_color: RgbColor,
    pub exp: f32,
}

pub struct HillsScene {
    terrain_data: Buffer<TerrainData>,
    ground_mesh: MeshObject,
    ground_renderer: StandardRenderer,
}

impl HillsScene {
    pub fn new(context: &mut CreationContext) -> Self {
        let mut ground_mesh = Mesh::new(4);
        primitives::quad(&mut ground_mesh.insert(), 800, 200);

        HillsScene {
            terrain_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            ground_mesh: MeshObject::new(context, &ground_mesh),
            ground_renderer: StandardRenderer::new(context, "terrain.vs", "terrain.ps"),
        }
    }

    pub fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        clouds: &mut CloudsRenderer,
        shadow_map: &mut ShadowMapRenderer,
        godray: &mut GodrayRenderer,
        properties: &[&[ClipPropertyValue]],
    ) {
        let terrain_center: Vector3 = prop(properties, 0, 0);
        let terrain_size: Vector3 = prop(properties, 0, 1);

        let light_map_pos: Vector3 = prop(properties, 1, 0);
        let light_map_x_clip: Vector2 = prop(properties, 1, 1);
        let light_map_y_clip: Vector2 = prop(properties, 1, 2);
        let light_map_z_clip: Vector2 = prop(properties, 1, 3);

        let fog_color = prop::<RgbaColor>(properties, 2, 0).premult();
        let fog_exp: f32 = prop(properties, 2, 1);
        let density: f32 = prop(properties, 2, 2);

        let cloud_y: f32 = prop(properties, 3, 0);
        let cloud_height: f32 = prop(properties, 3, 1);
        let map_offset: Vector3 = prop(properties, 3, 2);

        let sky_color = prop::<RgbaColor>(properties, 3, 3).premult();
        let scatter_color = prop::<RgbaColor>(properties, 3, 4).premult();

        let terrain_size = Vector3 {
            x: terrain_size.x.max(0.01),
            y: terrain_size.y.max(0.01),
            z: terrain_size.z.max(0.01),
        };

        let model_matrix = Matrix4::scale(terrain_size) * Matrix4::translate(terrain_center);

        // Render ground plane
        let ground_plane_perf = context.perf.start_gpu_str("render ground plane");
        self.terrain_data.upload(
            context.devcon,
            TerrainData {
                fog_color,
                exp: fog_exp,
            },
        );

        unsafe {
            (*context.devcon).PSSetConstantBuffers(4, 1, &self.terrain_data.ptr());
        }
        self.ground_renderer.render_start(
            context,
            &io.render_targets(),
            io.depth_map().depth_stencil_view_ptr(),
        );
        self.ground_mesh.render(context, model_matrix);
        self.ground_renderer.render_end(context);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(4, 1, &ptr::null_mut());
        }
        context.perf.end(ground_plane_perf);

        // Render clouds
        let clouds_perf = context.perf.start_gpu_str("clouds");
        clouds.render(
            context,
            CloudsData {
                map_offset,
                cloud_y,
                sky_color,
                cloud_height,
                scatter_color,
                cloud_opacity: 1.,
                light_direction: Vector3 {
                    x: 0.,
                    y: -1.,
                    z: 5.,
                }
                    .unit(),
            },
            io.world_pos_map_write(),
            io.write_output(),
            false,
        );
        context.perf.end(clouds_perf);

        // Render shadow map
        let shadow_map_perf = context.perf.start_gpu_str("shadow map");
        let (shadow_target, viewport, world_to_shadow_transform, shadow_map_state) =
            shadow_map.render_start(
                context,
                light_map_pos,
                light_map_x_clip,
                light_map_y_clip,
                light_map_z_clip,
            );
        self.ground_renderer
            .render_start(context, &[], shadow_target.depth_stencil_view_ptr());
        unsafe {
            (*context.devcon).RSSetViewports(1, &viewport);
        }
        self.ground_mesh.render(context, model_matrix);
        unsafe {
            (*context.devcon).RSSetViewports(0, ptr::null_mut());
        }
        self.ground_renderer.render_end(context);
        shadow_map.render_end(context, shadow_map_state);
        context.perf.end(shadow_map_perf);

        // Render godrays
        let godrays_perf = context.perf.start_gpu_str("godrays");
        godray.render(
            context,
            world_to_shadow_transform,
            density,
            400,
            4.,
            0.,
            io.world_pos_map_write(),
            shadow_map.shadow_map(),
            None,
            io.write_output(),
        );
        context.perf.end(godrays_perf);
    }
}
