use core::{mem, ptr};
use winapi::um::d3d11::{
    ID3D11Device, ID3D11RasterizerState, D3D11_CULL_NONE, D3D11_FILL_SOLID, D3D11_FILL_WIREFRAME,
    D3D11_RASTERIZER_DESC,
};

pub struct RasterState {
    state: *mut ID3D11RasterizerState,
}

impl RasterState {
    pub fn new_wireframe(device: *mut ID3D11Device) -> Self {
        let mut desc: D3D11_RASTERIZER_DESC = unsafe { mem::zeroed() };
        desc.FillMode = D3D11_FILL_WIREFRAME;
        desc.CullMode = D3D11_CULL_NONE;
        desc.DepthClipEnable = 1;

        let mut state = ptr::null_mut();
        check_err!(unsafe { (*device).CreateRasterizerState(&desc, &mut state) });

        RasterState { state }
    }

    pub fn new_no_cull(device: *mut ID3D11Device) -> Self {
        let mut desc: D3D11_RASTERIZER_DESC = unsafe { mem::zeroed() };
        desc.FillMode = D3D11_FILL_SOLID;
        desc.CullMode = D3D11_CULL_NONE;
        desc.DepthClipEnable = 1;

        let mut state = ptr::null_mut();
        check_err!(unsafe { (*device).CreateRasterizerState(&desc, &mut state) });

        RasterState { state }
    }

    pub fn ptr(&self) -> *mut ID3D11RasterizerState {
        self.state
    }
}

impl Drop for RasterState {
    fn drop(&mut self) {
        unsafe {
            (*self.state).Release();
        }
    }
}
