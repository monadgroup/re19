use super::common::PostRenderer;
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::texture::{RenderTarget2D, ShaderResource2D};
use core::ptr;
use winapi::um::d3d11::D3D11_BIND_CONSTANT_BUFFER;

#[derive(Clone, Copy)]
#[repr(C)]
struct ChromabData {
    chromab_amount: f32,
    grain_amount: f32,
}

pub struct ChromabRenderer {
    renderer: PostRenderer,
    data: Buffer<ChromabData>,
}

impl ChromabRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        ChromabRenderer {
            renderer: PostRenderer::new(context, "chromab.ps"),
            data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        chromab_amount: f32,
        grain_amount: f32,
        source: &dyn ShaderResource2D,
        target: &dyn RenderTarget2D,
    ) {
        self.data.upload(
            context.devcon,
            ChromabData {
                chromab_amount,
                grain_amount: grain_amount * context.viewport.width as f32 / 1920.,
            },
        );
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &self.data.ptr());
            (*context.devcon).PSSetShaderResources(0, 1, &source.shader_resource_ptr());
        }
        self.renderer.render(context, target, true, true);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
