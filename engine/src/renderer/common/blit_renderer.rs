use super::post_renderer::PostRenderer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::texture::{RenderTarget2D, ShaderResource2D};
use core::ptr;

pub struct BlitRenderer {
    renderer: PostRenderer,
}

impl BlitRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        BlitRenderer {
            renderer: PostRenderer::new(context, "blit.ps"),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        source: &dyn ShaderResource2D,
        target: &dyn RenderTarget2D,
        set_viewport: bool,
    ) {
        unsafe {
            (*context.devcon).PSSetShaderResources(0, 1, &source.shader_resource_ptr());
        }
        self.renderer.render(context, target, set_viewport, true);
        unsafe {
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
