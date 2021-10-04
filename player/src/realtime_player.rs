use crate::splash_screen::SplashScreen;
use crate::{config_window, exit_process};
use core::ffi::c_void;
use core::intrinsics::unreachable;
use core::mem::swap;
use core::{mem, ptr};
use engine::texture::{BackBuffer, RenderTarget2D};
use engine::viewport::Viewport;
use engine::{check_eq, check_ne, cstr};
use winapi::shared::dxgi::IDXGISwapChain;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, ID3D11Resource};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::timeapi::timeGetTime;
use winapi::um::wingdi::{DEVMODEA, DM_PELSHEIGHT, DM_PELSWIDTH};
use winapi::um::winuser::{
    AdjustWindowRect, ChangeDisplaySettingsA, CreateWindowExA, DefWindowProcA, DestroyWindow,
    DispatchMessageA, EnumDisplaySettingsA, GetSystemMetrics, LoadIconA, PeekMessageA,
    PostQuitMessage, RegisterClassA, ShowCursor, CDS_FULLSCREEN, CS_OWNDC, DISP_CHANGE_SUCCESSFUL,
    MAKEINTRESOURCEA, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, VK_ESCAPE, WM_CLOSE, WM_KEYDOWN,
    WM_QUIT, WNDCLASSA, WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_POPUP,
    WS_SYSMENU, WS_VISIBLE,
};

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

pub struct RealtimePlayerInitializer;

impl RealtimePlayerInitializer {
    pub fn open_window() -> Option<(HWND, Viewport, bool)> {
        unsafe {
            let inst = GetModuleHandleA(ptr::null_mut());
            let class_name = cstr!("Atlas");

            let config = match config_window::show() {
                Some(config) => config,
                None => return None,
            };

            // Create and show the window
            let mut window_class: WNDCLASSA = mem::zeroed();
            window_class.style = CS_OWNDC;
            window_class.lpfnWndProc = Some(wnd_proc);
            window_class.hInstance = inst;
            window_class.lpszClassName = class_name;
            window_class.hIcon = LoadIconA(inst, MAKEINTRESOURCEA(103));
            check_ne!(RegisterClassA(&window_class), 0);

            let dw_style = if config.is_fullscreen {
                let mut dm: DEVMODEA = mem::zeroed();
                dm.dmSize = mem::size_of::<DEVMODEA>() as _;
                dm.dmPelsWidth = config.display_width;
                dm.dmPelsHeight = config.display_height;
                dm.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT;

                check_eq!(
                    ChangeDisplaySettingsA(&mut dm, CDS_FULLSCREEN),
                    DISP_CHANGE_SUCCESSFUL
                );
                ShowCursor(0);
                WS_VISIBLE | WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN
            } else {
                WS_VISIBLE | WS_CAPTION | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_SYSMENU
            };

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: config.display_width as _,
                bottom: config.display_height as _,
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

            Some((
                h_wnd,
                Viewport {
                    width: config.display_width,
                    height: config.display_height,
                },
                config.is_prerender,
            ))
        }
    }

    pub fn start_load(
        device: *mut ID3D11Device,
        devcon: *mut ID3D11DeviceContext,
        window_viewport: Viewport,
        prerender_audio: bool,
    ) -> RealtimePlayerLoader {
        RealtimePlayerLoader::new(device, devcon, window_viewport, prerender_audio)
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

struct ProgressData<'a> {
    player_loader: &'a mut RealtimePlayerLoader,
    hwnd: HWND,
    swap_chain: *mut IDXGISwapChain,
    devcon: *mut ID3D11DeviceContext,
    target: &'a dyn RenderTarget2D,
}

extern "C" fn progress_callback(progress: f64, data: *mut c_void) {
    let progress_data = unsafe { &mut *(data as *mut ProgressData) };
    progress_data.player_loader.display_absolute_progress(
        progress_data.hwnd,
        progress_data.swap_chain,
        progress_data.devcon,
        progress_data.target,
        progress as f32 * 0.5,
    );
}

pub struct RealtimePlayerLoader {
    splash_screen: SplashScreen,
    prerender_audio: bool,
}

impl RealtimePlayerLoader {
    fn new(
        device: *mut ID3D11Device,
        devcon: *mut ID3D11DeviceContext,
        window_viewport: Viewport,
        prerender_audio: bool,
    ) -> Self {
        RealtimePlayerLoader {
            splash_screen: SplashScreen::new(device, devcon, window_viewport),
            prerender_audio,
        }
    }

    pub fn start(
        &mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
    ) {
        let prerender_audio = self.prerender_audio;

        struct ProgressData<'a> {
            player_loader: &'a mut RealtimePlayerLoader,
            hwnd: HWND,
            swap_chain: *mut IDXGISwapChain,
            devcon: *mut ID3D11DeviceContext,
            target: &'a dyn RenderTarget2D,
        }
        let mut data = ProgressData {
            player_loader: self,
            hwnd,
            swap_chain,
            devcon,
            target,
        };

        unsafe {
            wavesabre_sys::AudioInit(
                prerender_audio as u8,
                Some(progress_callback),
                &mut data as *mut ProgressData as *mut c_void,
            )
        };
    }

    fn display_absolute_progress(
        &mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
        progress: f32,
    ) {
        process_events(hwnd);
        self.splash_screen
            .render(swap_chain, devcon, target, progress, false);
    }

    pub fn display_progress(
        &mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
        progress: f32,
    ) {
        let remapped_progress = if self.prerender_audio {
            progress * 0.5 + 0.5
        } else {
            progress
        };

        self.display_absolute_progress(hwnd, swap_chain, devcon, target, remapped_progress);
    }

    pub fn finish(
        mut self,
        hwnd: HWND,
        swap_chain: *mut IDXGISwapChain,
        devcon: *mut ID3D11DeviceContext,
        target: &dyn RenderTarget2D,
    ) -> RealtimePlayerRunner {
        let fade_start_time = unsafe { timeGetTime() };

        loop {
            let passed_milliseconds = unsafe { timeGetTime() } - fade_start_time;
            let passed_seconds = passed_milliseconds as f32 / 1000.;

            process_events(hwnd);
            self.splash_screen
                .render(swap_chain, devcon, target, 1. + passed_seconds, true);

            if passed_seconds > 2. {
                break;
            }
        }

        RealtimePlayerRunner
    }
}

pub struct RealtimePlayerRunner;

impl RealtimePlayerRunner {
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
        unsafe {
            wavesabre_sys::AudioPlay();
        }

        loop {
            process_events(hwnd);

            let passed_seconds = unsafe { wavesabre_sys::AudioGetPos() };
            let passed_frames = (passed_seconds * framerate) as u32;

            if passed_frames > duration {
                exit_process(0);
            }

            f(passed_frames);

            unsafe {
                (*swap_chain).Present(1, 0);
            }
        }
    }
}
