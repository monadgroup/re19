use core::{mem, ptr, slice};
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Texture3D, D3D11_BIND_FLAG, D3D11_MAP_WRITE_DISCARD,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE3D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_IMMUTABLE,
};

pub struct Texture3D {
    texture: *mut ID3D11Texture3D,
    size: usize,
}

impl Texture3D {
    pub fn new(
        device: *mut ID3D11Device,
        width: u32,
        height: u32,
        depth: u32,
        mip_levels: u32,
        format: DXGI_FORMAT,
        bind_flag: D3D11_BIND_FLAG,
    ) -> Self {
        let tex_desc = D3D11_TEXTURE3D_DESC {
            Width: width,
            Height: height,
            Depth: depth,
            MipLevels: mip_levels,
            Format: format,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: bind_flag,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut tex_obj = ptr::null_mut();
        check_err!(unsafe { (*device).CreateTexture3D(&tex_desc, ptr::null(), &mut tex_obj) });

        Texture3D {
            texture: tex_obj,
            size: (width * height * depth) as usize,
        }
    }

    pub fn new_immutable(
        device: *mut ID3D11Device,
        width: u32,
        height: u32,
        depth: u32,
        mip_levels: u32,
        format: DXGI_FORMAT,
        bind_flag: D3D11_BIND_FLAG,
        data: &[f32],
    ) -> Self {
        let tex_desc = D3D11_TEXTURE3D_DESC {
            Width: width,
            Height: height,
            Depth: depth,
            MipLevels: mip_levels,
            Format: format,
            Usage: D3D11_USAGE_IMMUTABLE,
            BindFlags: bind_flag,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let subresource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: &data[0] as *const f32 as *const _,
            SysMemPitch: width * mem::size_of::<f32>() as u32,
            SysMemSlicePitch: width * height * mem::size_of::<f32>() as u32,
        };
        let mut tex_obj = ptr::null_mut();
        check_err!(unsafe {
            (*device).CreateTexture3D(&tex_desc, &subresource_data, &mut tex_obj)
        });

        Texture3D {
            texture: tex_obj,
            size: (width * height * depth) as usize,
        }
    }

    pub fn map(&self, devcon: *mut ID3D11DeviceContext) -> &mut [f32] {
        let mut mapped_resource = unsafe { mem::zeroed() };
        check_err!(unsafe {
            (*devcon).Map(
                self.texture as *mut _,
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                &mut mapped_resource,
            )
        });

        let float_ptr = mapped_resource.pData as *mut f32;
        unsafe { slice::from_raw_parts_mut(float_ptr, self.size) }
    }

    pub fn unmap(&self, devcon: *mut ID3D11DeviceContext) {
        unsafe {
            (*devcon).Unmap(self.texture as *mut _, 0);
        }
    }

    pub fn ptr(&self) -> *mut ID3D11Texture3D {
        self.texture
    }
}

impl Drop for Texture3D {
    fn drop(&mut self) {
        unsafe {
            (*self.texture).Release();
        }
    }
}
