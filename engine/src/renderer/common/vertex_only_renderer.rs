use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::resources::shader_manager::VertexKey;
use crate::vertex_layout::VertexLayout;
use core::ptr;
use winapi::shared::dxgiformat::{DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32_FLOAT};
use winapi::um::d3d11::{
    D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
    D3D11_VIEWPORT,
};

const ELEMENTS: [D3D11_INPUT_ELEMENT_DESC; 3] = [
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "POSITION\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "NORMAL\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "TEXCOORD\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: DXGI_FORMAT_R32G32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
];

pub struct VertexOnlyRenderer {
    layout: VertexLayout,
    shader: VertexKey,
}

impl VertexOnlyRenderer {
    pub fn new(context: &mut CreationContext, vertex_shader: &str) -> Self {
        let vertex_shader = context
            .shader_manager
            .load_shader(context.device, vertex_shader);
        let vertex_layout = VertexLayout::new(
            context.device,
            &context.shader_manager[vertex_shader],
            &ELEMENTS,
        );

        VertexOnlyRenderer {
            layout: vertex_layout,
            shader: vertex_shader,
        }
    }

    pub fn render_start(&self, context: &FrameContext) {
        unsafe {
            // bind IA
            (*context.devcon).IASetInputLayout(self.layout.ptr());

            // bind VS
            (*context.devcon).VSSetConstantBuffers(
                0,
                4,
                &[
                    context.common.frame_data_buffer.ptr(),
                    context.common.camera_buffer.ptr(),
                    context.common.object_buffer.ptr(),
                    context.common.light_buffer.ptr(),
                ][0],
            );
            (*context.devcon).VSSetShader(
                context.shader_manager[self.shader].get_shader(),
                ptr::null(),
                0,
            );

            // bind RS
            (*context.devcon).RSSetViewports(
                1,
                &D3D11_VIEWPORT {
                    TopLeftX: 0.,
                    TopLeftY: 0.,
                    Width: context.viewport.width as f32,
                    Height: context.viewport.height as f32,
                    MinDepth: 0.,
                    MaxDepth: 1.,
                },
            );
        }
    }

    pub fn render_end(&self, context: &FrameContext) {
        unsafe {
            // unbind IA
            (*context.devcon).IASetInputLayout(ptr::null_mut());

            // unbind VS
            (*context.devcon).VSSetConstantBuffers(
                0,
                4,
                &[
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ][0],
            );
            (*context.devcon).VSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }
}
