use crate::texture::ShaderResource2D;
use crate::viewport::Viewport;
use core::{mem, ptr};
use winapi::shared::dxgiformat::{
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R32_TYPELESS,
};
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::um::d3d11::{
    ID3D11DepthStencilView, ID3D11Device, ID3D11DeviceContext, ID3D11Resource,
    ID3D11ShaderResourceView, D3D11_BIND_DEPTH_STENCIL, D3D11_BIND_SHADER_RESOURCE,
    D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL, D3D11_DEPTH_STENCIL_VIEW_DESC,
    D3D11_DSV_DIMENSION_TEXTURE2D, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT,
};
use winapi::um::d3dcommon::D3D_SRV_DIMENSION_TEXTURE2D;

pub struct DepthStencil {
    depth_stencil_view: *mut ID3D11DepthStencilView,
    shader_view: *mut ID3D11ShaderResourceView,
}

impl DepthStencil {
    pub fn new(device: *mut ID3D11Device, viewport: Viewport) -> Self {
        let tex_desc = D3D11_TEXTURE2D_DESC {
            Width: viewport.width,
            Height: viewport.height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R32_TYPELESS,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_DEPTH_STENCIL | D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut tex_obj = ptr::null_mut();
        check_err!(unsafe { (*device).CreateTexture2D(&tex_desc, ptr::null(), &mut tex_obj) });

        let view_desc = D3D11_DEPTH_STENCIL_VIEW_DESC {
            Format: DXGI_FORMAT_D32_FLOAT,
            ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D,
            Flags: 0,
            u: unsafe { mem::zeroed() },
        };
        let mut view_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateDepthStencilView(
                tex_obj as *mut ID3D11Resource,
                &view_desc,
                &mut view_obj,
            )
        });

        let mut shader_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R32_FLOAT,
            ViewDimension: D3D_SRV_DIMENSION_TEXTURE2D,
            u: unsafe { mem::zeroed() },
        };
        unsafe { shader_desc.u.Texture2D_mut() }.MipLevels = 1;

        let mut shader_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(
                tex_obj as *mut ID3D11Resource,
                &shader_desc,
                &mut shader_obj,
            )
        });

        unsafe {
            (*tex_obj).Release();
        }

        DepthStencil {
            depth_stencil_view: view_obj,
            shader_view: shader_obj,
        }
    }

    pub fn depth_stencil_view_ptr(&self) -> *mut ID3D11DepthStencilView {
        self.depth_stencil_view
    }

    pub fn clear(&self, devcon: *mut ID3D11DeviceContext) {
        unsafe {
            (*devcon).ClearDepthStencilView(
                self.depth_stencil_view,
                D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL,
                1.,
                0xFF,
            )
        }
    }
}

impl ShaderResource2D for DepthStencil {
    fn shader_resource_ptr(&self) -> *mut ID3D11ShaderResourceView {
        self.shader_view
    }
}

impl Drop for DepthStencil {
    fn drop(&mut self) {
        unsafe {
            (*self.depth_stencil_view).Release();
            (*self.shader_view).Release();
        }
    }
}
