use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::camera::CameraBuffer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::{Matrix4, Vector2, Vector3, Vector4};
use crate::raster_state::RasterState;
use crate::renderer::common::PostRenderer;
use crate::shader_view::ShaderView;
use crate::texture::{AddressMode, DepthStencil, RenderTarget2D, Sampler, ShaderResource2D};
use crate::viewport::Viewport;
use core::ptr;
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_VIEWPORT,
};

const SHADOW_MAP_TEX_SIZE: (u32, u32) = (4096, 4096);

#[derive(Clone, Copy)]
#[repr(C)]
struct GodrayData {
    pub world_to_shadow_map: Matrix4,
    pub world_to_deep_shadow_map: Matrix4,
    pub density: f32,
    pub iterations: u32,
    pub step_length: f32,
    pub start_distance: f32,
    pub enable_deep_shadow_map: u32,
}

pub struct GodrayRenderer {
    godray_renderer: PostRenderer,
    godray_data: Buffer<GodrayData>,
    godray_blend: BlendState,
    smp: Sampler,
}

impl GodrayRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        GodrayRenderer {
            godray_renderer: PostRenderer::new(context, "godrays.ps"),
            godray_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            godray_blend: BlendState::new_dependent(
                context.device,
                false,
                BlendRenderTargetConfig::enabled(
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                ),
            ),
            smp: Sampler::new(
                context.device,
                D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressMode::Border(Vector4::default()),
            ),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        world_to_shadow_map: Matrix4,
        density: f32,
        iterations: u32,
        step_length: f32,
        start_distance: f32,
        world_pos: &dyn ShaderResource2D,
        shadow_map: &dyn ShaderResource2D,
        deep_shadow_map: Option<(Matrix4, &ShaderView)>,
        target: &dyn RenderTarget2D,
    ) {
        self.godray_data.upload(
            context.devcon,
            GodrayData {
                world_to_shadow_map,
                world_to_deep_shadow_map: deep_shadow_map
                    .map(|m| m.0)
                    .unwrap_or(Matrix4::default()),
                density,
                iterations,
                step_length,
                start_distance,
                enable_deep_shadow_map: deep_shadow_map.is_some() as u32,
            },
        );

        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[
                    context.common.camera_buffer.ptr(),
                    self.godray_data.ptr(),
                    context.common.light_buffer.ptr(),
                ][0],
            );
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[
                    world_pos.shader_resource_ptr(),
                    shadow_map.shader_resource_ptr(),
                    deep_shadow_map
                        .map(|m| m.1.ptr())
                        .unwrap_or(ptr::null_mut()),
                ][0],
            );
            (*context.devcon).PSSetSamplers(0, 1, &self.smp.sampler_state_ptr());
            (*context.devcon).OMSetBlendState(self.godray_blend.ptr(), &[1., 1., 1., 1.], 0xFFFFFF);
        }
        self.godray_renderer.render(context, target, true, false);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).PSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
        }
    }
}
