use core::ptr;
use engine::buffer::{Buffer, InitialData};
use engine::math::{RgbaColor, Vector2};
use engine::resources::shader::VertexShader;
use engine::texture::{
    create_gdi_tex, from_wmf, AddressMode, RenderTarget2D, Sampler, ShaderResource2D, Texture2D,
};
use engine::vertex_layout::VertexLayout;
use engine::viewport::Viewport;
use engine::{check_err, cstr};
use winapi::shared::dxgi::IDXGISwapChain;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32G32_FLOAT;
use winapi::shared::windef::RECT;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader, ID3D11VertexShader,
    D3D11_APPEND_ALIGNED_ELEMENT, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_VERTEX_BUFFER,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
    D3D11_VIEWPORT,
};
use winapi::um::d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use winapi::um::d3dcompiler::D3DCompile;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{
    CreateFontA, CreateSolidBrush, SelectObject, SetBkMode, SetTextCharacterExtra, SetTextColor,
    TextOutA, ANSI_CHARSET, ANTIALIASED_QUALITY, RGB, SYMBOL_CHARSET, TRANSPARENT,
};
use winapi::um::winuser::{DrawIconEx, FillRect, LoadIconA, MAKEINTRESOURCEA};

#[derive(Clone, Copy)]
#[repr(C)]
struct LoaderVertex {
    pub clip_pos: Vector2,
    pub tex_pos: Vector2,
}

pub struct SplashScreen {
    viewport: Viewport,

    splash_tex: Texture2D,
    splash_sampler: Sampler,
    vertex_shader: *mut ID3D11VertexShader,
    image_pixel_shader: *mut ID3D11PixelShader,
    bar_pixel_shader: *mut ID3D11PixelShader,

    vertex_layout: VertexLayout,
    tex_vertex_buffer: Buffer<LoaderVertex>,
    bar_vertex_buffer: Buffer<LoaderVertex>,
    image_data_buffer: Buffer<f32>,

    bh: f32,
    by: f32,
}

impl SplashScreen {
    pub fn new(
        device: *mut ID3D11Device,
        devcon: *mut ID3D11DeviceContext,
        viewport: Viewport,
    ) -> Self {
        unsafe {
            let inst = GetModuleHandleA(ptr::null_mut());
            let splash_size = Viewport {
                width: 634,
                height: 260,
            };
            let full_splash_height = splash_size.height + 200;
            let splash_tex = create_gdi_tex(
                device,
                devcon,
                splash_size,
                RgbaColor::new(1., 1., 1., 1.),
                |hdc| unsafe {
                    let brush = CreateSolidBrush(RGB(255, 255, 255));
                    FillRect(
                        hdc,
                        &RECT {
                            left: 0,
                            top: 0,
                            right: splash_size.width as i32,
                            bottom: splash_size.height as i32,
                        },
                        brush,
                    );

                    /*let h_icon = LoadIconA(inst, MAKEINTRESOURCEA(103));
                    DrawIconEx(
                        hdc,
                        0,
                        0,
                        h_icon as *mut _,
                        500,
                        500,
                        0,
                        ptr::null_mut(),
                        0x0003,
                    );*/
                    let font = CreateFontA(
                        195,
                        0,
                        0,
                        0,
                        800,
                        0,
                        0,
                        0,
                        ANSI_CHARSET,
                        0,
                        0,
                        ANTIALIASED_QUALITY,
                        0,
                        cstr!("Arial Black"),
                    );
                    SetBkMode(hdc, TRANSPARENT as _);
                    SetTextCharacterExtra(hdc, -15);
                    SelectObject(hdc, font as *mut _);
                    SetTextColor(hdc, 0x00000000);
                    TextOutA(hdc, -10, 0, cstr!("MONAD"), 5);
                    TextOutA(hdc, -10, 99, cstr!("EXPORTS"), 7);

                    let font_small = CreateFontA(
                        24,
                        0,
                        0,
                        0,
                        800,
                        0,
                        0,
                        0,
                        ANSI_CHARSET,
                        0,
                        0,
                        ANTIALIASED_QUALITY,
                        0,
                        cstr!("Arial Black"),
                    );
                    SelectObject(hdc, font_small as *mut _);
                    TextOutA(hdc, 618, 145, &0xA9u8 as *const u8 as *const i8, 1);
                },
            );
            let splash_sampler =
                Sampler::new(device, D3D11_FILTER_MIN_MAG_MIP_LINEAR, AddressMode::Clamp);

            let vertex_shader_src = include_str!("loader.vs");
            let bar_pixel_shader_src = include_str!("loader_bar.ps");
            let image_pixel_shader_src = include_str!("loader_image.ps");

            let mut vertex_shader_blob = ptr::null_mut();
            let mut vertex_shader = ptr::null_mut();
            check_err!(D3DCompile(
                vertex_shader_src.as_ptr() as *const _,
                vertex_shader_src.len(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                cstr!("main"),
                cstr!("vs_5_0"),
                0,
                0,
                &mut vertex_shader_blob,
                ptr::null_mut()
            ));
            check_err!((*device).CreateVertexShader(
                (*vertex_shader_blob).GetBufferPointer(),
                (*vertex_shader_blob).GetBufferSize(),
                ptr::null_mut(),
                &mut vertex_shader,
            ));

            let mut bar_pixel_shader_blob = ptr::null_mut();
            let mut bar_pixel_shader = ptr::null_mut();
            check_err!(D3DCompile(
                bar_pixel_shader_src.as_ptr() as *const _,
                bar_pixel_shader_src.len(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                cstr!("main"),
                cstr!("ps_5_0"),
                0,
                0,
                &mut bar_pixel_shader_blob,
                ptr::null_mut()
            ));
            check_err!((*device).CreatePixelShader(
                (*bar_pixel_shader_blob).GetBufferPointer(),
                (*bar_pixel_shader_blob).GetBufferSize(),
                ptr::null_mut(),
                &mut bar_pixel_shader,
            ));

            let mut image_pixel_shader_blob = ptr::null_mut();
            let mut image_pixel_shader = ptr::null_mut();
            check_err!(D3DCompile(
                image_pixel_shader_src.as_ptr() as *const _,
                image_pixel_shader_src.len(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                cstr!("main"),
                cstr!("ps_5_0"),
                0,
                0,
                &mut image_pixel_shader_blob,
                ptr::null_mut()
            ));
            check_err!((*device).CreatePixelShader(
                (*image_pixel_shader_blob).GetBufferPointer(),
                (*image_pixel_shader_blob).GetBufferSize(),
                ptr::null_mut(),
                &mut image_pixel_shader,
            ));

            let scale = (viewport.height as f32 / (full_splash_height as f32 + 100.)).min(1.);

            let sw = splash_size.width as f32 / viewport.width as f32 * scale;
            let sh = splash_size.height as f32 / viewport.height as f32 * scale;
            let sy = (full_splash_height - splash_size.height) as f32 / 2. / viewport.height as f32
                * scale;
            let image_data_buffer = Buffer::new_dynamic(
                device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            );
            let tex_vertex_buffer = Buffer::new_immutable(
                device,
                &[
                    LoaderVertex {
                        clip_pos: Vector2 { x: -sw, y: sh + sy },
                        tex_pos: Vector2 { x: 0., y: 0. },
                    },
                    LoaderVertex {
                        clip_pos: Vector2 { x: sw, y: -sh + sy },
                        tex_pos: Vector2 { x: 1., y: 1. },
                    },
                    LoaderVertex {
                        clip_pos: Vector2 {
                            x: -sw,
                            y: -sh + sy,
                        },
                        tex_pos: Vector2 { x: 0., y: 1. },
                    },
                    LoaderVertex {
                        clip_pos: Vector2 { x: -sw, y: sh + sy },
                        tex_pos: Vector2 { x: 0., y: 0. },
                    },
                    LoaderVertex {
                        clip_pos: Vector2 { x: sw, y: sh + sy },
                        tex_pos: Vector2 { x: 1., y: 0. },
                    },
                    LoaderVertex {
                        clip_pos: Vector2 { x: sw, y: -sh + sy },
                        tex_pos: Vector2 { x: 1., y: 1. },
                    },
                ],
                D3D11_BIND_VERTEX_BUFFER,
            );

            let bh = 50. / viewport.height as f32 * scale;
            let by = -(splash_size.height as f32 + 60.) as f32 / viewport.height as f32 * scale;
            let bar_vertex_buffer = Buffer::new_dynamic(
                device,
                InitialData::Uninitialized(6),
                D3D11_BIND_VERTEX_BUFFER,
            );
            let vertex_layout = VertexLayout::new(
                device,
                &VertexShader::new(vertex_shader, vertex_shader_blob),
                &[
                    D3D11_INPUT_ELEMENT_DESC {
                        SemanticName: cstr!("POSITION"),
                        SemanticIndex: 0,
                        Format: DXGI_FORMAT_R32G32_FLOAT,
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0,
                    },
                    D3D11_INPUT_ELEMENT_DESC {
                        SemanticName: cstr!("TEXCOORD"),
                        SemanticIndex: 0,
                        Format: DXGI_FORMAT_R32G32_FLOAT,
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0,
                    },
                ],
            );

            SplashScreen {
                viewport,

                splash_tex,
                splash_sampler,
                vertex_shader,
                image_pixel_shader,
                bar_pixel_shader,

                vertex_layout,
                tex_vertex_buffer,
                bar_vertex_buffer,
                image_data_buffer,

                bh,
                by,
            }
        }
    }

    pub fn render(
        &mut self,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
        progress: f32,
        wait_for_vblank: bool,
    ) {
        unsafe {
            let x_progress = progress.min(1.);
            let y_progress = (progress - 1.).max(0.);
            let top_y = self.bh + self.by - y_progress / 10.;
            let bottom_y = self.by - self.bh + y_progress / 10.;
            self.bar_vertex_buffer.do_map(devcon, |buffer| {
                let slice = buffer.slice_mut();
                slice[0] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: -x_progress,
                        y: top_y,
                    },
                    tex_pos: Vector2 { x: 0., y: 0. },
                };
                slice[1] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: x_progress,
                        y: bottom_y,
                    },
                    tex_pos: Vector2 { x: 1., y: 1. },
                };
                slice[2] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: -x_progress,
                        y: bottom_y,
                    },
                    tex_pos: Vector2 { x: 0., y: 1. },
                };

                slice[3] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: -x_progress,
                        y: top_y,
                    },
                    tex_pos: Vector2 { x: 0., y: 0. },
                };
                slice[4] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: x_progress,
                        y: top_y,
                    },
                    tex_pos: Vector2 { x: 1., y: 0. },
                };
                slice[5] = LoaderVertex {
                    clip_pos: Vector2 {
                        x: x_progress,
                        y: bottom_y,
                    },
                    tex_pos: Vector2 { x: 1., y: 1. },
                };
            });

            let alpha = 1. - y_progress / 2.;
            self.image_data_buffer.upload(devcon, alpha);

            target.clear(devcon, RgbaColor::new(alpha, alpha, alpha, 1.));
            (*devcon).IASetInputLayout(self.vertex_layout.ptr());
            (*devcon).PSSetSamplers(0, 1, &self.splash_sampler.sampler_state_ptr());
            (*devcon).PSSetShaderResources(0, 1, &self.splash_tex.shader_resource_ptr());
            (*devcon).PSSetConstantBuffers(0, 1, &self.image_data_buffer.ptr());
            (*devcon).OMSetRenderTargets(1, &target.target_view_ptr(), ptr::null_mut());
            (*devcon).IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            (*devcon).VSSetShader(self.vertex_shader, ptr::null(), 0);
            (*devcon).RSSetViewports(
                1,
                &D3D11_VIEWPORT {
                    TopLeftX: 0.,
                    TopLeftY: 0.,
                    Width: self.viewport.width as f32,
                    Height: self.viewport.height as f32,
                    MinDepth: 0.,
                    MaxDepth: 1.,
                },
            );

            (*devcon).IASetVertexBuffers(
                0,
                1,
                &self.tex_vertex_buffer.ptr(),
                &(self.tex_vertex_buffer.stride() as u32),
                &0,
            );
            (*devcon).PSSetShader(self.image_pixel_shader, ptr::null(), 0);
            (*devcon).Draw(self.tex_vertex_buffer.len() as u32, 0);

            (*devcon).IASetVertexBuffers(
                0,
                1,
                &self.bar_vertex_buffer.ptr(),
                &(self.bar_vertex_buffer.stride() as u32),
                &0,
            );
            (*devcon).PSSetShader(self.bar_pixel_shader, ptr::null(), 0);
            (*devcon).Draw(self.bar_vertex_buffer.len() as u32, 0);

            (*devcon).IASetInputLayout(ptr::null_mut());
            (*devcon).PSSetSamplers(0, 1, &ptr::null_mut());
            (*devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
            (*devcon).PSSetConstantBuffers(0, 1, &ptr::null_mut());
            (*devcon).OMSetRenderTargets(0, ptr::null(), ptr::null_mut());
            (*devcon).VSSetShader(ptr::null_mut(), ptr::null(), 0);
            (*devcon).IASetVertexBuffers(0, 1, &ptr::null_mut(), &0, &0);
            (*devcon).PSSetShader(ptr::null_mut(), ptr::null(), 0);

            // Pass 0 as first parameter to not wait for v-blank (because that slows down loading)
            (*swap_chain).Present(wait_for_vblank as u32, 0);
        }
    }
}
