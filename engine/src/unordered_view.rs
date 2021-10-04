use crate::math::RgbaColor;
use crate::texture::{Texture2D, Texture3D};
use core::{mem, ops, ptr};
use winapi::shared::dxgiformat::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN};
use winapi::um::d3d11::{
    ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11UnorderedAccessView,
    D3D11_UAV_DIMENSION_BUFFER, D3D11_UAV_DIMENSION_TEXTURE3D, D3D11_UNORDERED_ACCESS_VIEW_DESC,
};

pub struct UnorderedView {
    view: *mut ID3D11UnorderedAccessView,
}

impl UnorderedView {
    pub fn for_structured_buffer(
        device: *mut ID3D11Device,
        buffer: *mut ID3D11Buffer,
        range: ops::Range<u32>,
    ) -> Self {
        let mut desc = D3D11_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D11_UAV_DIMENSION_BUFFER,
            u: unsafe { mem::zeroed() },
        };

        let buffer_u = unsafe { desc.u.Buffer_mut() };
        buffer_u.FirstElement = range.start;
        buffer_u.NumElements = range.end - range.start;
        buffer_u.Flags = 0;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateUnorderedAccessView(buffer as *mut _, &desc, &mut view)
        });

        UnorderedView { view }
    }

    pub fn for_texture_2d(device: *mut ID3D11Device, texture: &Texture2D) -> Self {
        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateUnorderedAccessView(texture.ptr() as *mut _, ptr::null(), &mut view)
        });

        UnorderedView { view }
    }

    pub fn for_texture_3d(device: *mut ID3D11Device, texture: &Texture3D) -> Self {
        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateUnorderedAccessView(texture.ptr() as *mut _, ptr::null(), &mut view)
        });

        UnorderedView { view }
    }

    pub fn for_texture_3d_layer(
        device: *mut ID3D11Device,
        texture: &Texture3D,
        format: DXGI_FORMAT,
        layer: u32,
    ) -> Self {
        let mut desc = D3D11_UNORDERED_ACCESS_VIEW_DESC {
            Format: format,
            ViewDimension: D3D11_UAV_DIMENSION_TEXTURE3D,
            u: unsafe { mem::zeroed() },
        };
        let u = unsafe { desc.u.Texture3D_mut() };
        u.MipSlice = layer;
        u.WSize = !0;

        let mut view = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateUnorderedAccessView(texture.ptr() as *mut _, &desc, &mut view)
        });

        UnorderedView { view }
    }

    pub fn clear(&self, devcon: *mut ID3D11DeviceContext, color: RgbaColor) {
        unsafe {
            (*devcon).ClearUnorderedAccessViewFloat(self.ptr(), &color.into());
        }
    }

    pub fn ptr(&self) -> *mut ID3D11UnorderedAccessView {
        self.view
    }
}

impl Drop for UnorderedView {
    fn drop(&mut self) {
        unsafe {
            (*self.view).Release();
        }
    }
}
