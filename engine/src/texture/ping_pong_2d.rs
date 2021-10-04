use super::{RenderTarget2D, ShaderResource2D, Texture2D};
use crate::viewport::Viewport;
use core::mem;
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::um::d3d11::ID3D11Device;

pub struct PingPong2D {
    read_tex: Texture2D,
    write_tex: Texture2D,
}

impl PingPong2D {
    pub fn new(device: *mut ID3D11Device, viewport: Viewport, format: DXGI_FORMAT) -> Self {
        PingPong2D {
            read_tex: Texture2D::new(device, viewport, 1, format, 0, 0),
            write_tex: Texture2D::new(device, viewport, 1, format, 0, 0),
        }
    }

    pub fn swap_and_get(&mut self) -> (&dyn ShaderResource2D, &dyn RenderTarget2D) {
        mem::swap(&mut self.read_tex, &mut self.write_tex);
        (&self.read_tex, &self.write_tex)
    }

    pub fn get_read(&self) -> &Texture2D {
        &self.read_tex
    }

    pub fn get_write(&self) -> &Texture2D {
        &self.write_tex
    }
}
