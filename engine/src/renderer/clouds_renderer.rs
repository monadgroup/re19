use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::gbuffer::GBuffer;
use crate::math::{RgbColor, Vector3};
use crate::renderer::common::PostRenderer;
use crate::texture::{RenderTarget2D, ShaderResource2D};
use core::ptr;
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CloudsData {
    pub map_offset: Vector3,
    pub cloud_y: f32,
    pub sky_color: RgbColor,
    pub cloud_height: f32,
    pub scatter_color: RgbColor,
    pub cloud_opacity: f32,
    pub light_direction: Vector3,
}

pub struct CloudsRenderer {
    renderer: PostRenderer,
    data: Buffer<CloudsData>,
    blend: BlendState,
}

impl CloudsRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        CloudsRenderer {
            renderer: PostRenderer::new(context, "clouds.ps"),
            data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            blend: BlendState::new_dependent(
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
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        data: CloudsData,
        world_pos: &dyn ShaderResource2D,
        write_out: &dyn RenderTarget2D,
        defer_blend: bool,
    ) {
        self.data.upload(context.devcon, data);

        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[
                    context.common.camera_buffer.ptr(),
                    self.data.ptr(),
                    context.common.light_buffer.ptr(),
                ][0],
            );
            (*context.devcon).PSSetShaderResources(0, 1, &world_pos.shader_resource_ptr());

            if !defer_blend {
                (*context.devcon).OMSetBlendState(self.blend.ptr(), &[1., 1., 1., 1.], 0xFFFFFF);
            }
        }
        self.renderer.render(context, write_out, true, true);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());

            if defer_blend {
                (*context.devcon).OMSetBlendState(self.blend.ptr(), &[1., 1., 1., 1.], 0xFFFFFF);
            } else {
                (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
            }
        }
    }
}
