use super::RenderTarget2D;
use crate::viewport::Viewport;
use core::{mem, ptr};
use winapi::shared::dxgi::IDXGISwapChain;
use winapi::um::d3d11::{ID3D11Device, ID3D11RenderTargetView, ID3D11Resource, ID3D11Texture2D};
use winapi::Interface;

pub struct BackBuffer {
    texture: *mut ID3D11Texture2D,
    target_view: *mut ID3D11RenderTargetView,
}

impl BackBuffer {
    pub fn from_swapchain(device: *mut ID3D11Device, swapchain: *mut IDXGISwapChain) -> Self {
        let mut back_buffer: *mut ID3D11Texture2D = ptr::null_mut();
        check_err!(unsafe {
            (*swapchain).GetBuffer(
                0,
                &ID3D11Texture2D::uuidof(),
                &mut back_buffer as *mut *mut ID3D11Texture2D as *mut *mut _,
            )
        });
        let mut render_target = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateRenderTargetView(
                back_buffer as *mut ID3D11Resource,
                ptr::null(),
                &mut render_target,
            )
        });

        BackBuffer {
            texture: back_buffer,
            target_view: render_target,
        }
    }
}

impl RenderTarget2D for BackBuffer {
    fn target_view_ptr(&self) -> *mut ID3D11RenderTargetView {
        self.target_view
    }

    fn size(&self) -> Viewport {
        let mut desc = unsafe { mem::uninitialized() };
        unsafe { (*self.texture).GetDesc(&mut desc) };

        Viewport {
            width: desc.Width,
            height: desc.Height,
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            (*self.texture).Release();
            (*self.target_view).Release();
        }
    }
}
