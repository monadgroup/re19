use crate::texture::{Texture2D, Texture3D};
use core::{mem, ptr};
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11RenderTargetView, D3D11_RENDER_TARGET_VIEW_DESC,
    D3D11_RTV_DIMENSION_TEXTURE2D, D3D11_RTV_DIMENSION_TEXTURE3D,
};

pub struct TargetView {
    view: *mut ID3D11RenderTargetView,
}

impl TargetView {
    pub fn for_texture_2d_layer(
        device: *mut ID3D11Device,
        texture: &Texture2D,
        format: DXGI_FORMAT,
        layer: u32,
    ) -> Self {
        let mut desc = D3D11_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
            u: unsafe { mem::zeroed() },
        };
        let u = unsafe { desc.u.Texture2D_mut() };
        u.MipSlice = layer;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateRenderTargetView(texture.ptr() as *mut _, &desc, &mut view)
        });

        TargetView { view }
    }

    pub fn for_texture_3d(device: *mut ID3D11Device, texture: &Texture3D) -> Self {
        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateRenderTargetView(texture.ptr() as *mut _, ptr::null(), &mut view)
        });

        TargetView { view }
    }

    pub fn for_texture_3d_layer(
        device: *mut ID3D11Device,
        texture: &Texture3D,
        format: DXGI_FORMAT,
        layer: u32,
    ) -> Self {
        let mut desc = D3D11_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: D3D11_RTV_DIMENSION_TEXTURE3D,
            u: unsafe { mem::zeroed() },
        };
        let u = unsafe { desc.u.Texture3D_mut() };
        u.MipSlice = layer;
        u.WSize = !0;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateRenderTargetView(texture.ptr() as *mut _, &desc, &mut view)
        });

        TargetView { view }
    }

    pub fn ptr(&self) -> *mut ID3D11RenderTargetView {
        self.view
    }
}

impl Drop for TargetView {
    fn drop(&mut self) {
        unsafe {
            (*self.view).Release();
        }
    }
}
