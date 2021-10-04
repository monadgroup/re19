use core::{mem, ptr};
use winapi::um::d3d11::{ID3D11DepthStencilState, ID3D11Device, D3D11_DEPTH_STENCIL_DESC};

pub struct DepthState {
    state: *mut ID3D11DepthStencilState,
}

impl DepthState {
    pub fn enable(device: *mut ID3D11Device) -> Self {
        let mut desc: D3D11_DEPTH_STENCIL_DESC = unsafe { mem::zeroed() };
        desc.DepthEnable = 1;
        DepthState::new(device, &desc)
    }

    pub fn disable(device: *mut ID3D11Device) -> Self {
        DepthState::new(device, &unsafe {
            mem::zeroed::<D3D11_DEPTH_STENCIL_DESC>()
        })
    }

    fn new(device: *mut ID3D11Device, desc: &D3D11_DEPTH_STENCIL_DESC) -> Self {
        let mut state_ptr = ptr::null_mut();
        check_err!(unsafe { (*device).CreateDepthStencilState(desc, &mut state_ptr) });
        DepthState { state: state_ptr }
    }

    pub fn ptr(&self) -> *mut ID3D11DepthStencilState {
        self.state
    }
}

impl Drop for DepthState {
    fn drop(&mut self) {
        unsafe {
            (*self.state).Release();
        }
    }
}
