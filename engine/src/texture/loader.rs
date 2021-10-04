use super::{RenderTarget2D, Texture2D};
use crate::math::RgbaColor;
use crate::viewport::Viewport;
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::shared::windef::{HDC, RECT};
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, D3D11_RESOURCE_MISC_GDI_COMPATIBLE};
use winapi::um::wingdi::{DeleteEnhMetaFile, PlayEnhMetaFile, SetEnhMetaFileBits};

pub fn create_gdi_tex<F: FnOnce(HDC)>(
    device: *mut ID3D11Device,
    devcon: *mut ID3D11DeviceContext,
    viewport: Viewport,
    background: RgbaColor,
    func: F,
) -> Texture2D {
    let mut tex = Texture2D::new(
        device,
        viewport,
        1,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        0,
        D3D11_RESOURCE_MISC_GDI_COMPATIBLE,
    );
    tex.clear(devcon, background);
    tex.with_dc(func);
    tex
}

pub fn from_wmf(
    device: *mut ID3D11Device,
    devcon: *mut ID3D11DeviceContext,
    bytes: &[u8],
    size: Viewport,
    background: RgbaColor,
) -> Texture2D {
    create_gdi_tex(device, devcon, size, background, |hdc| unsafe {
        // todo: figure out why this isn't working (it'd be nice to use WMF instead of EMF)
        /*let meta_file = SetMetaFileBitsEx(bytes.len() as u32, &bytes[0]);
        debug_assert!(!meta_file.is_null());
        printf(cstr!("meta file: %p, last error: %i\n"), meta_file, GetLastError());
        check_ne!(PlayMetaFile(hdc, meta_file), 0);
        check_err!(DeleteMetaFile(meta_file));*/

        let meta_file = SetEnhMetaFileBits(bytes.len() as u32, &bytes[0]);
        check_ne!(
            PlayEnhMetaFile(
                hdc,
                meta_file,
                &RECT {
                    left: 0,
                    top: 0,
                    right: size.width as i32,
                    bottom: size.height as i32,
                }
            ),
            0
        );
        check_ne!(DeleteEnhMetaFile(meta_file), 0);
    })
}
