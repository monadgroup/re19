use super::Texture3D;
use crate::math::random::rand_float;
use alloc::vec::Vec;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32_FLOAT;
use winapi::um::d3d11::{ID3D11Device, D3D11_BIND_SHADER_RESOURCE};

pub fn noise_volume(device: *mut ID3D11Device, width: u32, height: u32, depth: u32) -> Texture3D {
    let pixels: Vec<_> = (0..(width * height * depth))
        .into_iter()
        .map(|_| rand_float(0., 1.))
        .collect();
    Texture3D::new_immutable(
        device,
        width,
        height,
        depth,
        1,
        DXGI_FORMAT_R32_FLOAT,
        D3D11_BIND_SHADER_RESOURCE,
        &pixels,
    )
}
