use crate::math::RgbaColor;
use crate::viewport::Viewport;
use winapi::um::d3d11::{ID3D11DeviceContext, ID3D11RenderTargetView};

pub trait RenderTarget2D {
    fn target_view_ptr(&self) -> *mut ID3D11RenderTargetView;

    fn size(&self) -> Viewport;

    fn clear(&self, devcon: *mut ID3D11DeviceContext, color: RgbaColor) {
        unsafe { (*devcon).ClearRenderTargetView(self.target_view_ptr(), &color.into()) }
    }
}
