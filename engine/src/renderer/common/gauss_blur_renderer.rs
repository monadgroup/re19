use super::post_renderer::PostRenderer;
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::Vector2;
use crate::viewport::Viewport;
use core::ptr;
use winapi::um::d3d11::{
    ID3D11RenderTargetView, ID3D11ShaderResourceView, D3D11_BIND_CONSTANT_BUFFER,
};

#[derive(Clone, Copy)]
#[repr(C)]
struct GaussBlurData {
    target_size: Vector2,
    direction: Vector2,
}

pub struct GaussBlurRenderer {
    renderer: PostRenderer,
    data_buffer: Buffer<GaussBlurData>,
}

impl GaussBlurRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        GaussBlurRenderer {
            renderer: PostRenderer::new(context, "gauss_blur.ps"),
            data_buffer: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
        }
    }

    // Blurs the source the specified number of times, and leaves the output in the write buffer of
    // the ping-pong texture.
    pub fn render(
        &mut self,
        context: &mut FrameContext,
        h_source: *mut ID3D11ShaderResourceView,
        h_target: *mut ID3D11RenderTargetView,
        v_source: *mut ID3D11ShaderResourceView,
        v_target: *mut ID3D11RenderTargetView,
        h_target_viewport: Viewport,
        v_target_viewport: Viewport,
        iterations: usize,
        size: Vector2,
    ) {
        for _ in 0..iterations {
            self.data_buffer.upload(
                context.devcon,
                GaussBlurData {
                    target_size: h_target_viewport.into(),
                    direction: Vector2 { x: 1., y: 0. } * size,
                },
            );
            self.call_render(context, h_source, h_target, h_target_viewport);

            self.data_buffer.upload(
                context.devcon,
                GaussBlurData {
                    target_size: v_target_viewport.into(),
                    direction: Vector2 { x: 0., y: 1. } * size,
                },
            );
            self.call_render(context, v_source, v_target, v_target_viewport);
        }
    }

    fn call_render(
        &mut self,
        context: &mut FrameContext,
        source: *mut ID3D11ShaderResourceView,
        target: *mut ID3D11RenderTargetView,
        viewport: Viewport,
    ) {
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &self.data_buffer.ptr());
            (*context.devcon).PSSetShaderResources(0, 1, &source);
        }
        self.renderer
            .render_start(context, &[target], None, Some(viewport), true);
        context.common.screen_quad.render(context.devcon);
        self.renderer.render_end(context, true);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
