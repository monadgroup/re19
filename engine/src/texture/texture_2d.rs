use super::{RenderTarget2D, ShaderResource2D};
use crate::viewport::Viewport;
use core::marker::PhantomData;
use core::{mem, ptr};
use winapi::shared::dxgi::IDXGISurface1;
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::shared::windef::HDC;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11RenderTargetView, ID3D11Resource, ID3D11ShaderResourceView,
    ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
    D3D11_RESOURCE_MISC_FLAG, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_IMMUTABLE,
};
use winapi::Interface;

pub struct Texture2D {
    viewport: Viewport,
    texture: *mut ID3D11Texture2D,
    target_view: *mut ID3D11RenderTargetView,
    shader_view: *mut ID3D11ShaderResourceView,
}

impl Texture2D {
    pub fn new(
        device: *mut ID3D11Device,
        viewport: Viewport,
        mip_levels: u32,
        format: DXGI_FORMAT,
        bind_flags: D3D11_BIND_FLAG,
        misc_flag: D3D11_RESOURCE_MISC_FLAG,
    ) -> Self {
        let tex_desc = D3D11_TEXTURE2D_DESC {
            Width: viewport.width,
            Height: viewport.height,
            MipLevels: mip_levels,
            ArraySize: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET | bind_flags,
            CPUAccessFlags: 0,
            MiscFlags: misc_flag,
        };
        let mut tex_obj = ptr::null_mut();
        check_err!(unsafe { (*device).CreateTexture2D(&tex_desc, ptr::null(), &mut tex_obj) });

        let mut target_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateRenderTargetView(
                tex_obj as *mut ID3D11Resource,
                ptr::null(),
                &mut target_obj,
            )
        });

        let mut shader_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(
                tex_obj as *mut ID3D11Resource,
                ptr::null(),
                &mut shader_obj,
            )
        });

        Texture2D {
            viewport,
            texture: tex_obj,
            target_view: target_obj,
            shader_view: shader_obj,
        }
    }

    pub fn new_immutable(
        device: *mut ID3D11Device,
        viewport: Viewport,
        format: DXGI_FORMAT,
        data: &[f32],
    ) -> Self {
        let tex_desc = D3D11_TEXTURE2D_DESC {
            Width: viewport.width,
            Height: viewport.height,
            MipLevels: 1,
            ArraySize: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_IMMUTABLE,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let subresource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: &data[0] as *const f32 as *const _,
            SysMemPitch: viewport.width * mem::size_of::<f32>() as u32,
            SysMemSlicePitch: 0,
        };
        let mut tex_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateTexture2D(&tex_desc, &subresource_data, &mut tex_obj)
        });

        let mut shader_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateShaderResourceView(
                tex_obj as *mut ID3D11Resource,
                ptr::null(),
                &mut shader_obj,
            )
        });

        Texture2D {
            viewport,
            texture: tex_obj,
            target_view: ptr::null_mut(),
            shader_view: shader_obj,
        }
    }

    pub fn dc(&mut self) -> TextureDC {
        TextureDC::new(self)
    }

    pub fn with_dc<F: FnOnce(HDC)>(&mut self, f: F) {
        let dc = self.dc();
        f(dc.hdc());
    }

    pub fn ptr(&self) -> *mut ID3D11Texture2D {
        self.texture
    }
}

impl RenderTarget2D for Texture2D {
    fn target_view_ptr(&self) -> *mut ID3D11RenderTargetView {
        self.target_view
    }

    fn size(&self) -> Viewport {
        self.viewport
    }
}

impl ShaderResource2D for Texture2D {
    fn shader_resource_ptr(&self) -> *mut ID3D11ShaderResourceView {
        self.shader_view
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            (*self.texture).Release();
            (*self.shader_view).Release();

            // target_view is null when created with new_immutable
            if !self.target_view.is_null() {
                (*self.target_view).Release();
            }
        }
    }
}

pub struct TextureDC<'tex> {
    viewport: Viewport,
    hdc: HDC,
    surface: *mut IDXGISurface1,
    _tex: PhantomData<&'tex mut Texture2D>,
}

impl<'tex> TextureDC<'tex> {
    fn new(tex: &'tex mut Texture2D) -> Self {
        let mut surface: *mut IDXGISurface1 = ptr::null_mut();
        check_err!(unsafe {
            (*tex.ptr()).QueryInterface(
                &IDXGISurface1::uuidof(),
                &mut surface as *mut *mut IDXGISurface1 as *mut _,
            )
        });
        let mut hdc = ptr::null_mut();
        check_err!(unsafe { (*surface).GetDC(1, &mut hdc) });

        TextureDC {
            viewport: tex.size(),
            hdc,
            surface,
            _tex: PhantomData,
        }
    }

    pub fn hdc(&self) -> HDC {
        self.hdc
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }
}

impl<'tex> Drop for TextureDC<'tex> {
    fn drop(&mut self) {
        check_err!(unsafe { (*self.surface).ReleaseDC(ptr::null_mut()) });
    }
}
