use crate::math::Vector4;
use core::ptr;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11SamplerState, D3D11_COMPARISON_ALWAYS, D3D11_FILTER, D3D11_FLOAT32_MAX,
    D3D11_SAMPLER_DESC, D3D11_TEXTURE_ADDRESS_BORDER, D3D11_TEXTURE_ADDRESS_CLAMP,
    D3D11_TEXTURE_ADDRESS_MIRROR, D3D11_TEXTURE_ADDRESS_MODE, D3D11_TEXTURE_ADDRESS_WRAP,
};

#[derive(Clone, Copy)]
pub enum AddressMode {
    Clamp,
    Border(Vector4),
    Wrap,
    Mirror,
}

impl AddressMode {
    pub fn as_d3d11_address_mode(self) -> D3D11_TEXTURE_ADDRESS_MODE {
        match self {
            AddressMode::Clamp => D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressMode::Border(_) => D3D11_TEXTURE_ADDRESS_BORDER,
            AddressMode::Wrap => D3D11_TEXTURE_ADDRESS_WRAP,
            AddressMode::Mirror => D3D11_TEXTURE_ADDRESS_MIRROR,
        }
    }

    pub fn border_color(self) -> Vector4 {
        match self {
            AddressMode::Border(color) => color,
            _ => Vector4::default(),
        }
    }
}

pub struct Sampler {
    sampler_state: *mut ID3D11SamplerState,
}

impl Sampler {
    pub fn new(device: *mut ID3D11Device, filter: D3D11_FILTER, address_mode: AddressMode) -> Self {
        let d3d_address_mode = address_mode.as_d3d11_address_mode();
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: filter,
            AddressU: d3d_address_mode,
            AddressV: d3d_address_mode,
            AddressW: d3d_address_mode,
            MipLODBias: 0.,
            MaxAnisotropy: 1,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            BorderColor: address_mode.border_color().into(),
            MinLOD: 0.,
            MaxLOD: D3D11_FLOAT32_MAX,
        };
        let mut sampler_obj = ptr::null_mut();
        check_err!(unsafe { (*device).CreateSamplerState(&sampler_desc, &mut sampler_obj) });

        Sampler {
            sampler_state: sampler_obj,
        }
    }

    pub fn sampler_state_ptr(&self) -> *mut ID3D11SamplerState {
        self.sampler_state
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            (*self.sampler_state).Release();
        }
    }
}
