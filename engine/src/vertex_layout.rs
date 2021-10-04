use crate::resources::shader::VertexShader;
use core::ptr;
use winapi::um::d3d11::{ID3D11Device, ID3D11InputLayout, D3D11_INPUT_ELEMENT_DESC};

pub struct VertexLayout {
    layout: *mut ID3D11InputLayout,
}

impl VertexLayout {
    pub fn new(
        device: *mut ID3D11Device,
        validate_shader: &VertexShader,
        elements: &[D3D11_INPUT_ELEMENT_DESC],
    ) -> Self {
        let mut input_layout = ptr::null_mut();
        let shader_blob = validate_shader.get_blob();
        check_err!(unsafe {
            (*device).CreateInputLayout(
                &elements[0],
                elements.len() as u32,
                (*shader_blob).GetBufferPointer(),
                (*shader_blob).GetBufferSize(),
                &mut input_layout,
            )
        });

        VertexLayout {
            layout: input_layout,
        }
    }

    pub fn ptr(&self) -> *mut ID3D11InputLayout {
        self.layout
    }
}

impl Drop for VertexLayout {
    fn drop(&mut self) {
        unsafe {
            (*self.layout).Release();
        }
    }
}
