use engine::buffer::{Buffer, InitialData};
use engine::camera::CameraBuffer;
use engine::clips::{SceneClip, SceneTarget};
use engine::object::ObjectBuffer;
use engine::resources::shader_manager::ShaderManager;
use engine::{CreationContext, DepthStencilTarget, FrameContext, RenderTarget2D, Viewport};
use path_abs::PathDir;
use std::path::Path;
use std::{mem, ptr};
use winapi::shared::dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD};
use winapi::shared::dxgiformat::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN};
use winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT;
use winapi::shared::windef::HWND;
use winapi::um::d3d11::{
    D3D11CreateDeviceAndSwapChain, D3D11_BIND_CONSTANT_BUFFER, D3D11_CREATE_DEVICE_DEBUG,
    D3D11_SDK_VERSION, D3D11_VIEWPORT,
};
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext};
use winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE;
use winapi::um::winuser::GetClientRect;

fn get_window_viewport(window: HWND) -> Viewport {
    let mut rect = unsafe { mem::zeroed() };
    check_ne!(unsafe { GetClientRect(window, &mut rect) }, 0);

    Viewport {
        width: rect.right as u32,
        height: rect.bottom as u32,
    }
}

pub struct ProjectRenderer {
    shader_manager: ShaderManager,
    window: HWND,
    device: *mut ID3D11Device,
    swap_chain: *mut IDXGISwapChain,
    devcon: *mut ID3D11DeviceContext,

    fb_target: Option<RenderTarget2D>,
    depth_target: DepthStencilTarget,
    camera_buffer: Buffer<CameraBuffer>,
    object_buffer: Buffer<ObjectBuffer>,

    last_viewport: Viewport,
}

impl ProjectRenderer {
    pub fn new(project_path: &Path, window: HWND) -> Self {
        let shader_path = project_path.join("shaders");

        let mut device = ptr::null_mut();
        let mut swap_chain = ptr::null_mut();
        let mut devcon = ptr::null_mut();

        let mut scd = unsafe { mem::zeroed::<DXGI_SWAP_CHAIN_DESC>() };
        scd.BufferCount = 2;
        scd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        scd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        scd.OutputWindow = window;
        scd.SampleDesc.Count = 1;
        scd.Windowed = 1;
        scd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;
        check_err!(unsafe {
            D3D11CreateDeviceAndSwapChain(
                ptr::null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                ptr::null_mut(),
                if cfg!(debug_assertions) {
                    D3D11_CREATE_DEVICE_DEBUG
                } else {
                    0
                },
                ptr::null_mut(),
                0,
                D3D11_SDK_VERSION,
                &scd,
                &mut swap_chain,
                &mut device,
                ptr::null_mut(),
                &mut devcon,
            )
        });

        let viewport = get_window_viewport(window);
        ProjectRenderer {
            shader_manager: ShaderManager::new(PathDir::new(shader_path).unwrap()),
            window,
            device,
            swap_chain,
            devcon,
            fb_target: Some(RenderTarget2D::new_from_swapchain(device, swap_chain)),
            depth_target: DepthStencilTarget::new(device, viewport),
            camera_buffer: Buffer::new_dynamic(
                device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            object_buffer: Buffer::new_dynamic(
                device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            last_viewport: viewport,
        }
    }

    pub fn get_creation_context(&mut self) -> CreationContext {
        CreationContext {
            device: self.device,
            devcon: self.devcon,
            shader_manager: &mut self.shader_manager,
        }
    }

    pub fn render_frame(&mut self, clip: &mut SceneClip) {
        self.shader_manager.update(self.device);

        let current_viewport = get_window_viewport(self.window);
        if current_viewport != self.last_viewport {
            self.fb_target = None;
            check_err!(unsafe {
                (*self.swap_chain).ResizeBuffers(
                    0,
                    current_viewport.width,
                    current_viewport.height,
                    DXGI_FORMAT_UNKNOWN,
                    0,
                )
            });
            self.fb_target = Some(RenderTarget2D::new_from_swapchain(
                self.device,
                self.swap_chain,
            ));
            self.depth_target = DepthStencilTarget::new(self.device, current_viewport);
            self.last_viewport = current_viewport;
        };
        let fb_target = self.fb_target.as_ref().unwrap();

        let vp = D3D11_VIEWPORT {
            Width: current_viewport.width as f32,
            Height: current_viewport.height as f32,
            TopLeftX: 0.,
            TopLeftY: 0.,
            MinDepth: 0.,
            MaxDepth: 1.,
        };
        unsafe {
            (*self.devcon).RSSetViewports(1, &vp);
        }

        fb_target.clear(self.devcon, [0., 0., 0., 0.].into());
        self.depth_target.clear(self.devcon);

        clip.render(
            &mut FrameContext {
                devcon: self.devcon,
                viewport: current_viewport,
                camera_buffer: &mut self.camera_buffer,
                object_buffer: &mut self.object_buffer,
                shader_manager: &self.shader_manager,
            },
            SceneTarget {
                color: &fb_target,
                depth: &self.depth_target,
            },
        );

        self.swap_buffers();
    }

    fn swap_buffers(&self) {
        unsafe {
            (*self.swap_chain).Present(1, 0);
        }
    }
}

impl Drop for ProjectRenderer {
    fn drop(&mut self) {
        unsafe {
            (*self.devcon).Release();
            (*self.swap_chain).Release();
            (*self.device).Release();
        }
    }
}
