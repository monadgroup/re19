use super::common::PostRenderer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::gbuffer::GBuffer;
use crate::texture::RenderTarget2D;
use core::ptr;

pub struct SkyboxRenderer {
    renderer: PostRenderer,
}

impl SkyboxRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        SkyboxRenderer {
            renderer: PostRenderer::new(context, "skybox.ps"),
        }
    }

    pub fn render(&mut self, context: &mut FrameContext, target: &GBuffer) {
        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[
                    context.common.camera_buffer.ptr(),
                    ptr::null_mut(),
                    context.common.light_buffer.ptr(),
                ][0],
            );
        }
        self.renderer.render_start(
            context,
            &[
                target.write_output().target_view_ptr(),
                target.normal_map().target_view_ptr(),
                target.world_pos_map_write().target_view_ptr(),
            ],
            None,
            Some(context.viewport),
            true,
        );

        context.common.screen_quad.render(context.devcon);

        unsafe {
            (*context.devcon).PSSetConstantBuffers(
                1,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
        }
        self.renderer.render_end(context, true);
    }
}
