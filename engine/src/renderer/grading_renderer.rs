use super::common::PostRenderer;
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::{RgbColor, Vector2, Vector3};
use crate::texture::{RenderTarget2D, ShaderResource2D};
use core::ptr;
use winapi::um::d3d11::D3D11_BIND_CONSTANT_BUFFER;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GradingParameters {
    pub vignette_offset: Vector2,
    pub exposure: f32,
    pub fade: f32,
    pub curve: Vector3,
    pub vignette_strength: f32,
    pub gradient_color_a: RgbColor,
    pub vignette_size: f32,
    pub gradient_color_b: RgbColor,
    pub vignette_power: f32,
    pub gradient_pos_a: Vector2,
    pub gradient_pos_b: Vector2,
    pub gradient_dry_wet: f32,

    pub tonemap_a: f32,
    pub tonemap_b: f32,
    pub tonemap_c: f32,
    pub tonemap_d: f32,
    pub tonemap_e: f32,
    pub tonemap_f: f32,
    pub tonemap_w: f32,
}

pub struct GradingRenderer {
    renderer: PostRenderer,
    parameters: Buffer<GradingParameters>,
}

impl GradingRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        GradingRenderer {
            renderer: PostRenderer::new(context, "grading.ps"),
            parameters: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        parameters: GradingParameters,
        source: &dyn ShaderResource2D,
        target: &dyn RenderTarget2D,
    ) {
        self.parameters.upload(context.devcon, parameters);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &self.parameters.ptr());
            (*context.devcon).PSSetShaderResources(0, 1, &source.shader_resource_ptr());
        }
        self.renderer.render(context, target, true, true);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
