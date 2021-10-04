use crate::texture::{Texture2D, Texture3D};
use core::{mem, ops, ptr};
use winapi::shared::dxgiformat::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN};
use winapi::um::d3d11::{
    ID3D11Buffer, ID3D11Device, ID3D11ShaderResourceView, D3D11_SHADER_RESOURCE_VIEW_DESC,
};
use winapi::um::d3dcommon::{
    D3D11_SRV_DIMENSION_BUFFER, D3D11_SRV_DIMENSION_TEXTURE2D, D3D11_SRV_DIMENSION_TEXTURE3D,
};

pub struct ShaderView {
    view: *mut ID3D11ShaderResourceView,
}

impl ShaderView {
    pub fn for_structured_buffer(
        device: *mut ID3D11Device,
        buffer: *mut ID3D11Buffer,
        range: ops::Range<u32>,
    ) -> Self {
        let mut desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D11_SRV_DIMENSION_BUFFER,
            u: unsafe { mem::zeroed() },
        };
        unsafe {
            let buffer_u = desc.u.Buffer_mut();
            *buffer_u.u1.FirstElement_mut() = range.start;
            *buffer_u.u2.NumElements_mut() = range.end - range.start;
        }

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(buffer as *mut _, &desc, &mut view)
        });

        ShaderView { view }
    }

    pub fn for_texture_2d_layer(
        device: *mut ID3D11Device,
        texture: &Texture2D,
        format: DXGI_FORMAT,
        layer: u32,
    ) -> Self {
        let mut desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
            u: unsafe { mem::zeroed() },
        };
        let u = unsafe { desc.u.Texture2D_mut() };
        u.MostDetailedMip = layer;
        u.MipLevels = 1;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(texture.ptr() as *mut _, &desc, &mut view)
        });

        ShaderView { view }
    }

    pub fn for_texture_3d(device: *mut ID3D11Device, texture: &Texture3D) -> Self {
        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(texture.ptr() as *mut _, ptr::null(), &mut view)
        });

        ShaderView { view }
    }

    pub fn for_texture_3d_layer(
        device: *mut ID3D11Device,
        texture: &Texture3D,
        format: DXGI_FORMAT,
        layer: u32,
    ) -> Self {
        let mut desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE3D,
            u: unsafe { mem::zeroed() },
        };
        let u = unsafe { desc.u.Texture3D_mut() };
        u.MostDetailedMip = layer;
        u.MipLevels = 1;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(texture.ptr() as *mut _, &desc, &mut view)
        });

        ShaderView { view }
    }

    pub fn ptr(&self) -> *mut ID3D11ShaderResourceView {
        self.view
    }
}

impl Drop for ShaderView {
    fn drop(&mut self) {
        unsafe {
            (*self.view).Release();
        }
    }
}
