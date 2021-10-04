#![no_std]
#![no_main]
#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
)]

#[macro_use]
extern crate alloc;

pub mod config_window;
mod d3d_include;
mod deserializer;
mod framedrop_player;
mod player_clip_map;
mod player_generator_map;
mod realtime_player;
mod splash_screen;
mod system_allocator;

use self::config_window::Config;
use self::deserializer::{deserialize_shaders, deserialize_timeline, Stream};
use self::player_clip_map::PlayerClipMap;
use self::player_generator_map::PlayerGeneratorMap;
use self::splash_screen::SplashScreen;
use crate::framedrop_player::FramedropPlayerInitializer;
use crate::realtime_player::RealtimePlayerInitializer;
use crate::system_allocator::SystemAllocator;
use alloc::vec::Vec;
use core::intrinsics::{abort, unreachable};
use core::panic::PanicInfo;
use core::{mem, ptr};
use engine::animation::clip::ActiveClipMap;
use engine::animation::clip::{ActiveClip, ClipPropertyValue, ClipReference};
use engine::animation::coallesce::coallesce_animations;
use engine::animation::timeline::{ClipSource, Timeline};
use engine::creation_context::CreationContext;
use engine::frame_context::{CommonData, FrameContext, FrameDataBuffer};
use engine::gbuffer::GBuffer;
use engine::math::Vector2;
use engine::renderer::RendererCollection;
use engine::resources::perf_table::PerfTable;
use engine::resources::shader_manager::ShaderManager;
use engine::texture::{
    create_gdi_tex, AddressMode, BackBuffer, RenderTarget2D, Sampler, ShaderResource2D,
};
use engine::viewport::Viewport;
use engine::{check_eq, check_err, check_ne, cstr};
use winapi::shared::dxgi::{DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD};
use winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM;
use winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::ntdef::LPSTR;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::d3d11::{
    D3D11CreateDeviceAndSwapChain, ID3D11DeviceContext, D3D11_SDK_VERSION, D3D11_VIEWPORT,
};
use winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::timeapi::timeGetTime;
use winapi::um::winuser::{
    AdjustWindowRect, ChangeDisplaySettingsA, CreateWindowExA, DefWindowProcA, DestroyWindow,
    DispatchMessageA, DrawIcon, DrawIconEx, EnumDisplaySettingsA, GetSystemMetrics, LoadIconA,
    LoadImageA, PeekMessageA, PostQuitMessage, RegisterClassA, SetFocus, SetForegroundWindow,
    ShowCursor, CDS_FULLSCREEN, CS_OWNDC, DISP_CHANGE_SUCCESSFUL, IMAGE_ICON, LR_LOADTRANSPARENT,
    MAKEINTRESOURCEA, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, VK_ESCAPE, WM_CLOSE, WM_KEYDOWN,
    WM_QUIT, WNDCLASSA, WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_POPUP,
    WS_SYSMENU, WS_VISIBLE,
};

// note: should match framerate in tool
const FRAMERATE: f64 = 60.;

extern "C" {
    fn printf(format: *const i8, ...) -> i32;
}

#[global_allocator]
static ALLOC: SystemAllocator = SystemAllocator;

#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &PanicInfo) -> ! {
    if cfg!(debug_assertions) {
        let panic_str = format!("{}\0", info);
        unsafe {
            printf(cstr!("%s\n"), panic_str.as_ptr());
            abort();
        }
    } else {
        unsafe {
            unreachable();
        }
    }
}

#[alloc_error_handler]
fn error_handler(_: core::alloc::Layout) -> ! {
    if cfg!(debug_assertions) {
        unsafe { abort() }
    } else {
        unsafe { unreachable() }
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

pub fn exit_process(exit_code: u32) -> ! {
    unsafe {
        ExitProcess(exit_code);
        unreachable()
    }
}

fn render_frame(
    passed_frames: u32,
    devcon: *mut ID3D11DeviceContext,
    player_clip_map: &mut PlayerClipMap,
    timeline: &mut Timeline,
    common: &mut CommonData,
    shader_manager: &ShaderManager,
    renderer_collection: &mut RendererCollection,
    gbuffer: &mut GBuffer,
    viewport: Viewport,
) {
    player_clip_map.update(&timeline, passed_frames);
    coallesce_animations(&timeline, player_clip_map);

    let mut generator_map = Vec::new();
    generator_map.reserve(timeline.tracks.len());
    for track in &mut timeline.tracks {
        let clip = &mut track.clips[0];
        let gen = match &mut clip.source {
            ClipSource::Generator(gen) => Some(gen.as_mut()),
            _ => None,
        };
        generator_map.push(gen);
    }

    // update the frame data buffer
    common.frame_data = FrameDataBuffer {
        viewport: Vector2 {
            x: viewport.width as f32,
            y: viewport.height as f32,
        },
        seed: passed_frames as f32 / FRAMERATE as f32,
    };
    common.frame_data_buffer.upload(devcon, common.frame_data);

    let mut player_generator_map = PlayerGeneratorMap::new(generator_map);
    for active_clip in player_clip_map.active_clips() {
        player_generator_map.take(
            active_clip.reference.clip_id() as usize,
            |generator, map| {
                let mut frame_context = FrameContext {
                    devcon,
                    delta_seconds: 0., // todo
                    viewport,
                    shader_manager: &shader_manager,
                    clip_map: map,
                    common,
                    perf: &mut PerfTable::new(),
                };

                let prop_refs: Vec<_> = active_clip
                    .properties
                    .iter()
                    .map(|props| props as &[ClipPropertyValue])
                    .collect();
                generator.update(
                    gbuffer,
                    &mut frame_context,
                    renderer_collection,
                    active_clip.local_time,
                    &prop_refs,
                );
            },
        );
    }
}

type PlayerInitializer = RealtimePlayerInitializer;

#[no_mangle]
pub unsafe extern "C" fn WinMainCRTStartup(
    _inst: HINSTANCE,
    _prev_inst: HINSTANCE,
    _cmd_line: LPSTR,
    _cmd_show: i32,
) -> i32 {
    engine::math::random::seed_rand(0x1337b012);

    let (h_wnd, window_viewport, prerender_audio) = match PlayerInitializer::open_window() {
        Some(v) => v,
        None => exit_process(0),
    };

    let window_width = window_viewport.width as f32;
    let window_height = window_viewport.height as f32;

    // Determine the viewport we want
    let aspect_ratio: f32 = 2560. / 1080.;
    let viewport = if aspect_ratio * window_height > window_width {
        Viewport {
            width: window_width as u32,
            height: (window_width / aspect_ratio) as u32,
        }
    } else {
        Viewport {
            width: (window_height * aspect_ratio) as u32,
            height: window_height as u32,
        }
    };

    // Setup D3D11
    let mut device = ptr::null_mut();
    let mut swap_chain = ptr::null_mut();
    let mut devcon = ptr::null_mut();

    let mut scd: DXGI_SWAP_CHAIN_DESC = mem::zeroed();
    scd.BufferCount = 2;
    scd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    scd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    scd.OutputWindow = h_wnd;
    scd.SampleDesc.Count = 1;
    scd.Windowed = 1;
    scd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    check_err!(D3D11CreateDeviceAndSwapChain(
        ptr::null_mut(),
        D3D_DRIVER_TYPE_HARDWARE,
        ptr::null_mut(),
        0,
        ptr::null_mut(),
        0,
        D3D11_SDK_VERSION,
        &scd,
        &mut swap_chain,
        &mut device,
        ptr::null_mut(),
        &mut devcon,
    ));

    let letterbox_scale = window_viewport.width as f32 / viewport.width as f32;
    let letterbox_scale =
        if letterbox_scale * viewport.height as f32 > window_viewport.height as f32 {
            window_viewport.height as f32 / viewport.height as f32
        } else {
            letterbox_scale
        };
    let render_size = Vector2 {
        x: viewport.width as f32 * letterbox_scale,
        y: viewport.height as f32 * letterbox_scale,
    };
    let render_pos = Vector2 {
        x: window_viewport.width as f32 / 2. - render_size.x / 2.,
        y: window_viewport.height as f32 / 2. - render_size.y / 2.,
    };

    let back_buffer = BackBuffer::from_swapchain(device, swap_chain);
    let mut loader =
        PlayerInitializer::start_load(device, devcon, window_viewport, prerender_audio);

    SetForegroundWindow(h_wnd);
    SetFocus(h_wnd);
    loader.display_progress(h_wnd, swap_chain, devcon, &back_buffer, 0.);
    loader.start(h_wnd, swap_chain, devcon, &back_buffer);

    let mut data_stream = Stream::new(include_bytes!("../../project/data.blob"));

    // Load the shaders
    let (loaded_shaders, entry_points) =
        deserialize_shaders(&mut data_stream, device, &mut |progress| {
            loader.display_progress(h_wnd, swap_chain, devcon, &back_buffer, progress / 3.);
        });
    let mut shader_manager = ShaderManager::new(&loaded_shaders, entry_points);
    let mut creation_context = CreationContext {
        device,
        devcon,
        shader_manager: &mut shader_manager,
        viewport,
    };

    let mut common = CommonData::new(&mut creation_context);
    // todo: progress for RendererCollection creation
    let mut renderer_collection = RendererCollection::new(&mut creation_context);
    let mut gbuffer = GBuffer::new(device, viewport);

    let (project_duration, mut timeline) =
        deserialize_timeline(&mut data_stream, &mut creation_context, &mut |progress| {
            loader.display_progress(
                h_wnd,
                swap_chain,
                devcon,
                &back_buffer,
                1. / 3. + progress / 3.,
            );
        });

    let mut player_clip_map = PlayerClipMap::new(timeline.tracks.len());

    // Pre-render the start of each clip
    let clip_start_times: Vec<_> = timeline
        .tracks
        .iter()
        .filter_map(|track| {
            let clip = &track.clips[0];
            if clip.source.is_generator() {
                Some(clip)
            } else {
                None
            }
        })
        .scan(-1, |last_start_time, clip| {
            let last_start = *last_start_time;
            *last_start_time = clip.offset_frames as i32;

            Some((last_start, clip))
        })
        .filter_map(|(last_start_time, clip)| {
            if clip.offset_frames as i32 != last_start_time {
                Some(clip.offset_frames)
            } else {
                None
            }
        })
        .collect();
    let clip_start_time_len = clip_start_times.len();
    for (index, start_time) in clip_start_times.into_iter().enumerate() {
        render_frame(
            start_time,
            devcon,
            &mut player_clip_map,
            &mut timeline,
            &mut common,
            &shader_manager,
            &mut renderer_collection,
            &mut gbuffer,
            viewport,
        );

        let progress = index as f32 / clip_start_time_len as f32;
        loader.display_progress(
            h_wnd,
            swap_chain,
            devcon,
            &back_buffer,
            2. / 3. + progress / 3.,
        );
    }

    let runner = loader.finish(h_wnd, swap_chain, devcon, &back_buffer);

    runner.play(
        h_wnd,
        project_duration * 2,
        FRAMERATE * 2.,
        devcon,
        gbuffer.write_output().ptr() as *mut _,
        swap_chain,
        |passed_frames| {
            render_frame(
                passed_frames,
                devcon,
                &mut player_clip_map,
                &mut timeline,
                &mut common,
                &shader_manager,
                &mut renderer_collection,
                &mut gbuffer,
                viewport,
            );

            // Render letterboxed
            (*devcon).RSSetViewports(
                1,
                &D3D11_VIEWPORT {
                    TopLeftX: render_pos.x,
                    TopLeftY: render_pos.y,
                    Width: render_size.x,
                    Height: render_size.y,
                    MinDepth: 0.,
                    MaxDepth: 1.,
                },
            );

            // Copy the gbuffer output to the screen and display
            renderer_collection.blit.render(
                &mut FrameContext {
                    devcon,
                    delta_seconds: 0., // todo
                    viewport,
                    shader_manager: &shader_manager,
                    clip_map: &mut PlayerGeneratorMap::new(Vec::new()),
                    common: &mut common,
                    perf: &mut PerfTable::new(),
                },
                gbuffer.write_output(),
                &back_buffer,
                false,
            );
        },
    );
}

#[no_mangle]
pub static _fltused: i32 = 1;
