use super::common::PostRenderer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::texture::{RenderTarget2D, ShaderResource2D};
use core::ptr;

pub struct FxaaRenderer {
    renderer: PostRenderer,
}

impl FxaaRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        FxaaRenderer {
            renderer: PostRenderer::new(context, "fxaa.ps"),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        source: &dyn ShaderResource2D,
        target: &dyn RenderTarget2D,
    ) {
        unsafe {
            (*context.devcon).PSSetShaderResources(0, 1, &source.shader_resource_ptr());
        }
        self.renderer.render(context, target, true, true);
        unsafe {
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
