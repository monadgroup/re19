use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::resources::shader_manager::{PixelKey, VertexKey};
use crate::texture::{AddressMode, DepthStencil, RenderTarget2D, Sampler};
use crate::vertex_layout::VertexLayout;
use crate::viewport::Viewport;
use core::ptr;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32G32_FLOAT;
use winapi::um::d3d11::{
    ID3D11RenderTargetView, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
    D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_VIEWPORT,
};

const ELEMENTS: [D3D11_INPUT_ELEMENT_DESC; 1] = [D3D11_INPUT_ELEMENT_DESC {
    SemanticName: "POSITION\0".as_ptr() as *const i8,
    SemanticIndex: 0,
    Format: DXGI_FORMAT_R32G32_FLOAT,
    InputSlot: 0,
    AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
    InstanceDataStepRate: 0,
}];

pub struct PostRenderer {
    vertex_layout: VertexLayout,
    vertex_shader: VertexKey,
    pixel_shader: PixelKey,
    sampler: Sampler,
}

impl PostRenderer {
    pub fn new(context: &mut CreationContext, pixel_shader: &str) -> Self {
        let vertex_shader = context
            .shader_manager
            .load_shader(context.device, "post.vs");
        let pixel_shader = context
            .shader_manager
            .load_shader(context.device, pixel_shader);
        let vertex_layout = VertexLayout::new(
            context.device,
            &context.shader_manager[vertex_shader],
            &ELEMENTS,
        );
        let sampler = Sampler::new(
            context.device,
            D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressMode::Clamp,
        );

        PostRenderer {
            vertex_shader,
            pixel_shader,
            vertex_layout,
            sampler,
        }
    }

    pub fn render(
        &mut self,
        context: &mut FrameContext,
        target: &dyn RenderTarget2D,
        set_viewport: bool,
        set_sampler: bool,
    ) {
        self.render_start(
            context,
            &[target.target_view_ptr()],
            None,
            if set_viewport {
                Some(target.size())
            } else {
                None
            },
            set_sampler,
        );
        context.common.screen_quad.render(context.devcon);
        self.render_end(context, set_sampler);
    }

    pub fn render_start(
        &mut self,
        context: &mut FrameContext,
        targets: &[*mut ID3D11RenderTargetView],
        depth_target: Option<&DepthStencil>,
        viewport: Option<Viewport>,
        set_sampler: bool,
    ) {
        unsafe {
            // bind IA
            (*context.devcon).IASetInputLayout(self.vertex_layout.ptr());

            // bind VS
            (*context.devcon).VSSetConstantBuffers(0, 1, &context.common.frame_data_buffer.ptr());
            (*context.devcon).VSSetShader(
                context.shader_manager[self.vertex_shader].get_shader(),
                ptr::null(),
                0,
            );

            // bind PS
            (*context.devcon).PSSetConstantBuffers(0, 1, &context.common.frame_data_buffer.ptr());
            if set_sampler {
                (*context.devcon).PSSetSamplers(0, 1, &self.sampler.sampler_state_ptr());
            }
            (*context.devcon).PSSetShader(
                context.shader_manager[self.pixel_shader].get_shader(),
                ptr::null(),
                0,
            );

            // bind OM
            if !targets.is_empty() {
                (*context.devcon).OMSetRenderTargets(
                    targets.len() as u32,
                    &targets[0],
                    depth_target
                        .map(|depth| depth.depth_stencil_view_ptr())
                        .unwrap_or(ptr::null_mut()),
                );
            }

            // bind RS
            if let Some(viewport) = viewport {
                (*context.devcon).RSSetViewports(
                    1,
                    &D3D11_VIEWPORT {
                        TopLeftX: 0.,
                        TopLeftY: 0.,
                        Width: viewport.width as f32,
                        Height: viewport.height as f32,
                        MinDepth: 0.,
                        MaxDepth: 1.,
                    },
                );
            }
        }
    }

    pub fn render_end(&self, context: &mut FrameContext, clear_sampler: bool) {
        unsafe {
            // unbind IA
            (*context.devcon).IASetInputLayout(ptr::null_mut());

            // unbind VS
            (*context.devcon).VSSetConstantBuffers(0, 1, &ptr::null_mut());
            (*context.devcon).VSSetShader(ptr::null_mut(), ptr::null(), 0);

            // unbind PS
            (*context.devcon).PSSetConstantBuffers(0, 1, &ptr::null_mut());
            if clear_sampler {
                (*context.devcon).PSSetSamplers(0, 1, &ptr::null_mut());
            }
            (*context.devcon).PSSetShader(ptr::null_mut(), ptr::null(), 0);

            // unbind OM
            (*context.devcon).OMSetRenderTargets(0, ptr::null_mut(), ptr::null_mut());
        }
    }
}
