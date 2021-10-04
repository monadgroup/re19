use crate::exit_process;
use core::intrinsics::unreachable;
use core::{mem, ptr};
use engine::texture::RenderTarget2D;
use engine::viewport::Viewport;
use engine::{check_ne, cstr};
use winapi::shared::dxgi::IDXGISwapChain;
use winapi::shared::minwindef::{LPARAM, LRESULT, MAX_PATH, UINT, WPARAM};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, ID3D11Resource};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::processenv::GetCurrentDirectoryA;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::winnt::{HRESULT, LPCSTR, LPSTR};
use winapi::um::winuser::{
    AdjustWindowRect, ChangeDisplaySettingsA, CreateWindowExA, DefWindowProcA, DestroyWindow,
    DispatchMessageA, GetSystemMetrics, PeekMessageA, PostQuitMessage, RegisterClassA, ShowCursor,
    CS_OWNDC, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, VK_ESCAPE, WM_CLOSE, WM_KEYDOWN, WM_QUIT,
    WNDCLASSA, WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_POPUP, WS_SYSMENU,
    WS_VISIBLE,
};

#[repr(u32)]
enum D3DX11_IMAGE_FILE_FORMAT {
    BMP = 0,
    JPG = 1,
    PNG = 3,
    DDS = 4,
    TIFF = 10,
    GIF = 11,
    WMP = 12,
}

extern "C" {
    fn printf(format: *const i8, ...) -> i32;
}

extern "system" {
    fn D3DX11SaveTextureToFileA(
        pContext: *mut ID3D11DeviceContext,
        pSrcTexture: *mut ID3D11Resource,
        DestFormat: D3DX11_IMAGE_FILE_FORMAT,
        pDestFile: LPCSTR,
    ) -> HRESULT;

    fn PathCombineA(pszDest: LPSTR, pszDir: LPCSTR, pszFile: LPCSTR) -> LPSTR;
}

pub struct FramedropPlayerInitializer;

const RENDER_VIEWPORT: Viewport = Viewport {
    width: 5120,
    height: 2160,
};
const RENDER_FRAMERATE: f64 = 24.;

unsafe extern "system" fn wnd_proc(
    h_wnd: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if u_msg == WM_CLOSE || u_msg == WM_KEYDOWN && w_param == VK_ESCAPE as usize {
        PostQuitMessage(0);
        0
    } else {
        DefWindowProcA(h_wnd, u_msg, w_param, l_param)
    }
}

impl FramedropPlayerInitializer {
    pub fn open_window() -> Option<(HWND, Viewport)> {
        unsafe {
            let inst = GetModuleHandleA(ptr::null_mut());
            let class_name = cstr!("Atlas");

            let mut window_class: WNDCLASSA = mem::zeroed();
            window_class.style = CS_OWNDC;
            window_class.lpfnWndProc = Some(wnd_proc);
            window_class.hInstance = inst;
            window_class.lpszClassName = class_name;
            check_ne!(RegisterClassA(&window_class), 0);

            let dw_style = WS_VISIBLE | WS_CAPTION | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_SYSMENU;

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: RENDER_VIEWPORT.width as _,
                bottom: RENDER_VIEWPORT.height as _,
            };
            AdjustWindowRect(&mut rect, dw_style, 0);
            let h_wnd = CreateWindowExA(
                WS_EX_APPWINDOW,
                class_name,
                class_name,
                dw_style,
                GetSystemMetrics(SM_CXSCREEN) - rect.right + rect.left >> 1,
                GetSystemMetrics(SM_CYSCREEN) - rect.bottom + rect.top >> 1,
                rect.right - rect.left,
                rect.bottom - rect.top,
                ptr::null_mut(),
                ptr::null_mut(),
                inst,
                ptr::null_mut(),
            );

            Some((h_wnd, RENDER_VIEWPORT))
        }
    }

    pub fn start_load(
        device: *mut ID3D11Device,
        devcon: *mut ID3D11DeviceContext,
        window_viewport: Viewport,
    ) -> FramedropPlayerLoader {
        FramedropPlayerLoader
    }
}

fn process_events(h_wnd: HWND) {
    unsafe {
        let mut msg = mem::uninitialized();
        while PeekMessageA(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                DestroyWindow(h_wnd);
                ChangeDisplaySettingsA(ptr::null_mut(), 0);
                ShowCursor(1);
                exit_process(0);
            }

            DispatchMessageA(&msg);
        }
    }
}

pub struct FramedropPlayerLoader;

impl FramedropPlayerLoader {
    pub fn start(&self) {}

    pub fn display_progress(
        &mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
        progress: f32,
    ) {
        unsafe { printf(cstr!("Progress: %f%%\n"), progress as f64 * 100.) };
    }

    pub fn finish(
        mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
    ) -> FramedropPlayerRunner {
        FramedropPlayerRunner
    }
}

pub struct FramedropPlayerRunner;

impl FramedropPlayerRunner {
    pub fn play<F: FnMut(u32)>(
        self,
        hwnd: HWND,
        duration: u32,
        framerate: f64,
        context: *mut ID3D11DeviceContext,
        tex: *mut ID3D11Resource,
        swap_chain: *mut IDXGISwapChain,
        mut f: F,
    ) -> ! {
        let mut base_directory: [i8; MAX_PATH] = unsafe { mem::zeroed() };
        unsafe {
            GetCurrentDirectoryA(MAX_PATH as u32, &mut base_directory[0]);
        }

        let mut last_render_frame = -1;

        for project_frame in 0..duration {
            let passed_seconds = project_frame as f64 / framerate;
            let render_frame = (passed_seconds * RENDER_FRAMERATE) as i32;
            if render_frame == last_render_frame {
                continue;
            }
            last_render_frame = render_frame;

            process_events(hwnd);

            f(project_frame);

            let mut path: [i8; MAX_PATH] = unsafe { mem::zeroed() };
            unsafe {
                PathCombineA(
                    &mut path[0],
                    &base_directory[0],
                    format!("frame-{}.png\0", render_frame).as_ptr() as *const _,
                );
            }

            unsafe {
                D3DX11SaveTextureToFileA(context, tex, D3DX11_IMAGE_FILE_FORMAT::PNG, &path[0]);

                printf(cstr!("Frame %i (%fs)\n"), render_frame, passed_seconds);

                (*swap_chain).Present(0, 0);
            }
        }

        exit_process(0);
    }
}
