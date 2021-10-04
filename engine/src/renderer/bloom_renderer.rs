use super::common::{BlitRenderer, GaussBlurRenderer, PostRenderer};
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::Vector2;
use crate::texture::PingPong2D;
use crate::texture::RenderTarget2D;
use crate::texture::ShaderResource2D;
use core::ptr;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT;
use winapi::um::d3d11::D3D11_BIND_CONSTANT_BUFFER;

const BLUR_PASSES: usize = 6;
const BLUR_ITERATIONS: usize = 2;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExtractData {
    pub multiplier: f32,
    pub bias: f32,
    pub power: f32,
    pub amount: f32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CompositeData {
    pub shape: f32,
    pub multiplier: f32,
    pub bias: f32,
    pub power: f32,
    pub amount: f32,
}

pub struct BloomRenderer {
    blur_renderer: GaussBlurRenderer,
    ping_pongs: [PingPong2D; BLUR_PASSES],
    extract_renderer: PostRenderer,
    blit_renderer: BlitRenderer,
    composite_renderer: PostRenderer,
    extract_data: Buffer<ExtractData>,
    composite_data: Buffer<CompositeData>,
}

impl BloomRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        BloomRenderer {
            blur_renderer: GaussBlurRenderer::new(context),
            ping_pongs: [
                PingPong2D::new(
                    context.device,
                    context.viewport / 2,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
                PingPong2D::new(
                    context.device,
                    context.viewport / 4,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
                PingPong2D::new(
                    context.device,
                    context.viewport / 8,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
                PingPong2D::new(
                    context.device,
                    context.viewport / 16,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
                PingPong2D::new(
                    context.device,
                    context.viewport / 32,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
                PingPong2D::new(
                    context.device,
                    context.viewport / 64,
                    DXGI_FORMAT_R32G32B32A32_FLOAT,
                ),
            ],
            extract_renderer: PostRenderer::new(context, "bloom_extract.ps"),
            blit_renderer: BlitRenderer::new(context),
            composite_renderer: PostRenderer::new(context, "bloom_composite.ps"),
            extract_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            composite_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        source: &dyn ShaderResource2D,
        target: &dyn RenderTarget2D,
        size: Vector2,
        extract_data: ExtractData,
        composite_data: CompositeData,
    ) {
        let blur_perf = context.perf.start_gpu_str("blur");
        let mut last_source = source;

        for (pass_index, ping_pong) in self.ping_pongs.iter_mut().enumerate() {
            if pass_index == 0 {
                // Use the extract shader while downscaling
                self.extract_data.upload(context.devcon, extract_data);
                unsafe {
                    (*context.devcon).PSSetConstantBuffers(1, 1, &self.extract_data.ptr());
                    (*context.devcon).PSSetShaderResources(
                        0,
                        1,
                        &last_source.shader_resource_ptr(),
                    );
                }
                self.extract_renderer
                    .render(context, ping_pong.get_write(), true, true);
                unsafe {
                    (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
                    (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
                }
            } else {
                // Just use a blit shader
                self.blit_renderer
                    .render(context, last_source, ping_pong.get_write(), true);
            }

            self.blur_renderer.render(
                context,
                ping_pong.get_write().shader_resource_ptr(),
                ping_pong.get_read().target_view_ptr(),
                ping_pong.get_read().shader_resource_ptr(),
                ping_pong.get_write().target_view_ptr(),
                ping_pong.get_read().size(),
                ping_pong.get_write().size(),
                BLUR_ITERATIONS,
                size,
            );
            last_source = ping_pong.get_write();
        }
        context.perf.end(blur_perf);

        let composite_perf = context.perf.start_gpu_str("composite");
        self.composite_data.upload(context.devcon, composite_data);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &self.composite_data.ptr());
            (*context.devcon).PSSetShaderResources(
                0,
                BLUR_PASSES as u32 + 1,
                &[
                    source.shader_resource_ptr(),
                    self.ping_pongs[0].get_write().shader_resource_ptr(),
                    self.ping_pongs[1].get_write().shader_resource_ptr(),
                    self.ping_pongs[2].get_write().shader_resource_ptr(),
                    self.ping_pongs[3].get_write().shader_resource_ptr(),
                    self.ping_pongs[4].get_write().shader_resource_ptr(),
                    self.ping_pongs[5].get_write().shader_resource_ptr(),
                ][0],
            );
        }
        self.composite_renderer.render(context, target, true, true);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(
                0,
                BLUR_PASSES as u32 + 1,
                &[
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ][0],
            );
        }
        context.perf.end(composite_perf);
    }
}
