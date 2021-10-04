use super::VertexOnlyRenderer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::resources::shader_manager::PixelKey;
use core::ptr;
use winapi::um::d3d11::{ID3D11DepthStencilView, ID3D11RenderTargetView};

pub struct StandardRenderer {
    vertex_renderer: VertexOnlyRenderer,
    pixel_shader: PixelKey,
}

impl StandardRenderer {
    pub fn new(context: &mut CreationContext, vertex_shader: &str, pixel_shader: &str) -> Self {
        let vertex_renderer = VertexOnlyRenderer::new(context, vertex_shader);
        let pixel_shader = context
            .shader_manager
            .load_shader(context.device, pixel_shader);

        StandardRenderer {
            vertex_renderer,
            pixel_shader,
        }
    }

    pub fn render_start(
        &self,
        context: &FrameContext,
        targets: &[*mut ID3D11RenderTargetView],
        depth_target: *mut ID3D11DepthStencilView,
    ) {
        self.vertex_renderer.render_start(context);

        unsafe {
            // bind PS
            (*context.devcon).PSSetConstantBuffers(
                0,
                4,
                &[
                    context.common.frame_data_buffer.ptr(),
                    context.common.camera_buffer.ptr(),
                    context.common.object_buffer.ptr(),
                    context.common.light_buffer.ptr(),
                ][0],
            );
            (*context.devcon).PSSetShader(
                context.shader_manager[self.pixel_shader].get_shader(),
                ptr::null(),
                0,
            );

            // bind OM
            //if !targets.is_empty() {
            (*context.devcon).OMSetRenderTargets(
                targets.len() as u32,
                if targets.is_empty() {
                    ptr::null()
                } else {
                    &targets[0]
                },
                depth_target,
            );
            //}
        }
    }

    pub fn render_end(&self, context: &FrameContext) {
        self.vertex_renderer.render_end(context);

        unsafe {
            // unbind PS
            (*context.devcon).PSSetConstantBuffers(
                0,
                4,
                &[
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ][0],
            );
            (*context.devcon).PSSetShader(ptr::null_mut(), ptr::null(), 0);

            // unbind OM
            (*context.devcon).OMSetRenderTargets(0, ptr::null_mut(), ptr::null_mut());
        }
    }
}
