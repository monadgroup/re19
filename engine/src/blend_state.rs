use core::{mem, ptr};
use winapi::um::d3d11::{
    ID3D11BlendState, ID3D11Device, D3D11_BLEND, D3D11_BLEND_DESC, D3D11_BLEND_OP,
    D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_RENDER_TARGET_BLEND_DESC,
};

#[derive(Clone, Copy)]
pub struct BlendRenderTargetConfig {
    blend_desc: D3D11_RENDER_TARGET_BLEND_DESC,
}

impl BlendRenderTargetConfig {
    pub fn disabled() -> Self {
        BlendRenderTargetConfig {
            blend_desc: unsafe { mem::zeroed() },
        }
    }

    pub fn enabled(
        src_blend: D3D11_BLEND,
        dest_blend: D3D11_BLEND,
        blend_op: D3D11_BLEND_OP,
        src_blend_alpha: D3D11_BLEND,
        dest_blend_alpha: D3D11_BLEND,
        blend_op_alpha: D3D11_BLEND_OP,
    ) -> Self {
        BlendRenderTargetConfig {
            blend_desc: D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: 1,
                SrcBlend: src_blend,
                DestBlend: dest_blend,
                BlendOp: blend_op,
                SrcBlendAlpha: src_blend_alpha,
                DestBlendAlpha: dest_blend_alpha,
                BlendOpAlpha: blend_op_alpha,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL as u8,
            },
        }
    }
}

pub struct BlendState {
    state: *mut ID3D11BlendState,
}

impl BlendState {
    pub fn new_dependent(
        device: *mut ID3D11Device,
        alpha_to_coverage: bool,
        config: BlendRenderTargetConfig,
    ) -> Self {
        let mut desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: alpha_to_coverage as i32,
            IndependentBlendEnable: 0,
            RenderTarget: unsafe { mem::zeroed() },
        };
        desc.RenderTarget[0] = config.blend_desc;
        BlendState::new(device, &desc)
    }

    pub fn new_independent(
        device: *mut ID3D11Device,
        alpha_to_coverage: bool,
        configs: &[BlendRenderTargetConfig],
    ) -> Self {
        let mut desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: alpha_to_coverage as i32,
            IndependentBlendEnable: 1,
            RenderTarget: unsafe { mem::zeroed() },
        };
        for (index, config) in configs.into_iter().enumerate() {
            desc.RenderTarget[index] = config.blend_desc;
        }
        BlendState::new(device, &desc)
    }

    fn new(device: *mut ID3D11Device, desc: &D3D11_BLEND_DESC) -> Self {
        let mut state_ptr = ptr::null_mut();
        check_err!(unsafe { (*device).CreateBlendState(desc, &mut state_ptr) });

        BlendState { state: state_ptr }
    }

    pub fn ptr(&self) -> *mut ID3D11BlendState {
        self.state
    }
}

impl Drop for BlendState {
    fn drop(&mut self) {
        unsafe {
            (*self.state).Release();
        }
    }
}
