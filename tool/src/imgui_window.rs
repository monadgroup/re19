use field_offset::offset_of;
use imgui_sys::{
    igCreateContext, igGetCurrentContext, igGetDrawData, igGetIO, igGetMainViewport,
    igGetMouseCursor, igIsAnyMouseDown, igNewFrame, igRender, ImDrawData, ImDrawIdx, ImDrawVert,
    ImFontAtlas_GetTexDataAsRGBA32, ImGuiBackendFlags, ImGuiConfigFlags, ImGuiIO_AddInputCharacter,
    ImGuiKey, ImGuiMouseCursor, ImVec2,
};
use std::f32;
use std::ffi::CStr;
use std::os::raw::{c_int, c_uchar, c_ushort, c_void};
use std::{mem, ptr};
use winapi::shared::dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD};
use winapi::shared::dxgiformat::{
    DXGI_FORMAT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN,
};
use winapi::shared::dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT};
use winapi::shared::minwindef::{LOWORD, LPARAM, LPCVOID, LRESULT, UINT, WPARAM};
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::{HWND, POINT};
use winapi::um::d3d11::{
    D3D11CreateDeviceAndSwapChain, ID3D11BlendState, ID3D11Buffer, ID3D11ClassInstance,
    ID3D11DepthStencilState, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView, ID3D11Resource,
    ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader,
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_SHADER_RESOURCE,
    D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_OP_ADD,
    D3D11_BLEND_SRC_ALPHA, D3D11_BLEND_ZERO, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL,
    D3D11_COMPARISON_ALWAYS, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_DEBUG, D3D11_CULL_NONE,
    D3D11_DEPTH_STENCIL_DESC, D3D11_DEPTH_WRITE_MASK_ALL, D3D11_FILL_SOLID,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_PRIMITIVE_TOPOLOGY,
    D3D11_RASTERIZER_DESC, D3D11_RECT, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC,
    D3D11_SDK_VERSION, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_STENCIL_OP_KEEP,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_WRAP, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT, D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE,
};
use winapi::um::d3d11sdklayers::{ID3D11Debug, ID3D11InfoQueue};
use winapi::um::d3dcommon::{
    D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D11_SRV_DIMENSION_TEXTURE2D, D3D_DRIVER_TYPE_HARDWARE,
};
use winapi::um::d3dcompiler::D3DCompile;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use winapi::um::winuser::{
    ClientToScreen, DefWindowProcA, DispatchMessageA, GetCapture, GetClientRect, GetCursorPos,
    GetForegroundWindow, GetKeyState, IsChild, LoadCursorA, PeekMessageA, PostQuitMessage,
    ReleaseCapture, ScreenToClient, SetCapture, SetCursor, SetCursorPos, TranslateMessage,
    GET_WHEEL_DELTA_WPARAM, HTCLIENT, IDC_ARROW, IDC_HAND, IDC_IBEAM, IDC_SIZEALL, IDC_SIZENESW,
    IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, PM_REMOVE, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN,
    VK_END, VK_ESCAPE, VK_HOME, VK_INSERT, VK_LEFT, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN,
    VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP, WHEEL_DELTA, WM_CHAR, WM_CLOSE, WM_KEYDOWN,
    WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETCURSOR, WM_SYSKEYDOWN, WM_SYSKEYUP,
};
use winapi::Interface;

struct BackupState {
    scissor_rects_count: u32,
    viewports_count: u32,
    scissor_rects: [D3D11_RECT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as usize],
    viewports: [D3D11_VIEWPORT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as usize],
    raster_state: *mut ID3D11RasterizerState,
    blend_state: *mut ID3D11BlendState,
    blend_factor: [f32; 4],
    sample_mask: u32,
    stencil_ref: u32,
    depth_stencil_state: *mut ID3D11DepthStencilState,
    ps_shader_resource: *mut ID3D11ShaderResourceView,
    ps_sampler: *mut ID3D11SamplerState,
    ps: *mut ID3D11PixelShader,
    vs: *mut ID3D11VertexShader,
    ps_instances_count: u32,
    vs_instances_count: u32,
    ps_instances: [*mut ID3D11ClassInstance; 256],
    vs_instances: [*mut ID3D11ClassInstance; 256],
    primitive_topology: D3D11_PRIMITIVE_TOPOLOGY,
    index_buffer: *mut ID3D11Buffer,
    vertex_buffer: *mut ID3D11Buffer,
    vs_constant_buffer: *mut ID3D11Buffer,
    index_buffer_offset: u32,
    vertex_buffer_stride: u32,
    vertex_buffer_offset: u32,
    index_buffer_format: DXGI_FORMAT,
    input_layout: *mut ID3D11InputLayout,
}

#[repr(C)]
struct VertexConstantBuffer {
    mvp: [[f32; 4]; 4],
}

pub struct D3DResources {
    device: *mut ID3D11Device,
    devcon: *mut ID3D11DeviceContext,
    main_render_target: *mut ID3D11RenderTargetView,
    swap_chain: *mut IDXGISwapChain,
    input_layout: *mut ID3D11InputLayout,
    vertex_shader: *mut ID3D11VertexShader,
    pixel_shader: *mut ID3D11PixelShader,
    font_texture_view: *mut ID3D11ShaderResourceView,
    sampler: *mut ID3D11SamplerState,
    blend_state: *mut ID3D11BlendState,
    depth_stencil_state: *mut ID3D11DepthStencilState,
    rasterizer_state: *mut ID3D11RasterizerState,
    vertex_constant_buffer: *mut ID3D11Buffer,
    vertex_buffer: *mut ID3D11Buffer,
    index_buffer: *mut ID3D11Buffer,
    vertex_buffer_size: usize,
    index_buffer_size: usize,
}

impl D3DResources {
    fn new(window: HWND) -> Self {
        // setup the D3D device
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

        assert_eq!(
            unsafe {
                D3D11CreateDeviceAndSwapChain(
                    ptr::null_mut(),
                    D3D_DRIVER_TYPE_HARDWARE,
                    ptr::null_mut(),
                    D3D11_CREATE_DEVICE_DEBUG,
                    ptr::null_mut(),
                    0,
                    D3D11_SDK_VERSION,
                    &scd,
                    &mut swap_chain,
                    &mut device,
                    ptr::null_mut(),
                    &mut devcon,
                )
            },
            0
        );

        // setup the debug interface
        let mut debug_interface: *mut ID3D11Debug = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).QueryInterface(
                    &ID3D11Debug::uuidof(),
                    &mut debug_interface as *mut *mut ID3D11Debug as *mut _,
                )
            },
            0
        );
        let mut info_queue: *mut ID3D11InfoQueue = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).QueryInterface(
                    &ID3D11InfoQueue::uuidof(),
                    &mut info_queue as *mut *mut ID3D11InfoQueue as *mut _,
                )
            },
            0
        );
        /*unsafe {
            (*info_queue).SetBreakOnSeverity(D3D11_MESSAGE_SEVERITY_CORRUPTION, 1);
            (*info_queue).SetBreakOnSeverity(D3D11_MESSAGE_SEVERITY_ERROR, 1);
        }*/

        let (vertex_shader, input_layout, vertex_constant_buffer) =
            D3DResources::create_vertex_shader(device);
        let pixel_shader = D3DResources::create_pixel_shader(device);

        // create the blending setup
        let mut blend_desc = unsafe { mem::zeroed::<D3D11_BLEND_DESC>() };
        blend_desc.AlphaToCoverageEnable = 0;
        blend_desc.RenderTarget[0] = D3D11_RENDER_TARGET_BLEND_DESC {
            BlendEnable: 1,
            SrcBlend: D3D11_BLEND_SRC_ALPHA,
            DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
            BlendOp: D3D11_BLEND_OP_ADD,
            SrcBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
            DestBlendAlpha: D3D11_BLEND_ZERO,
            BlendOpAlpha: D3D11_BLEND_OP_ADD,
            RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL as u8,
        };
        let mut blend_state = ptr::null_mut();
        unsafe {
            (*device).CreateBlendState(&blend_desc, &mut blend_state);
        }

        // create the rasterizer setup
        let mut rasterizer_desc = unsafe { mem::zeroed::<D3D11_RASTERIZER_DESC>() };
        rasterizer_desc.FillMode = D3D11_FILL_SOLID;
        rasterizer_desc.CullMode = D3D11_CULL_NONE;
        rasterizer_desc.ScissorEnable = 1;
        rasterizer_desc.DepthClipEnable = 1;
        let mut rasterizer_state = ptr::null_mut();
        unsafe {
            (*device).CreateRasterizerState(&rasterizer_desc, &mut rasterizer_state);
        }

        // create the depth-stencil setup
        let mut depth_stencil_desc = unsafe { mem::zeroed::<D3D11_DEPTH_STENCIL_DESC>() };
        depth_stencil_desc.DepthEnable = 0;
        depth_stencil_desc.DepthWriteMask = D3D11_DEPTH_WRITE_MASK_ALL;
        depth_stencil_desc.DepthFunc = D3D11_COMPARISON_ALWAYS;
        depth_stencil_desc.StencilEnable = 0;
        depth_stencil_desc.FrontFace.StencilFailOp = D3D11_STENCIL_OP_KEEP;
        depth_stencil_desc.FrontFace.StencilDepthFailOp = D3D11_STENCIL_OP_KEEP;
        depth_stencil_desc.FrontFace.StencilPassOp = D3D11_STENCIL_OP_KEEP;
        depth_stencil_desc.FrontFace.StencilFunc = D3D11_COMPARISON_ALWAYS;
        depth_stencil_desc.BackFace = depth_stencil_desc.FrontFace;
        let mut depth_stencil_state = ptr::null_mut();
        unsafe {
            (*device).CreateDepthStencilState(&depth_stencil_desc, &mut depth_stencil_state);
        }

        let font_texture_view = D3DResources::create_font_texture(device);
        unsafe {
            (*(*igGetIO()).fonts).tex_id = font_texture_view as *mut _;
        }

        // create texture sampler
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
            MipLODBias: 0.,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            MinLOD: 0.,
            MaxLOD: 0.,
            MaxAnisotropy: 0,
            BorderColor: unsafe { mem::zeroed() },
        };
        let mut sampler = ptr::null_mut();
        unsafe {
            (*device).CreateSamplerState(&sampler_desc, &mut sampler);
        }

        let main_render_target = D3DResources::create_render_target(swap_chain, device);

        D3DResources {
            device,
            devcon,
            main_render_target,
            swap_chain,
            input_layout,
            vertex_shader,
            pixel_shader,
            font_texture_view,
            sampler,
            blend_state,
            depth_stencil_state,
            rasterizer_state,
            vertex_constant_buffer,
            vertex_buffer: ptr::null_mut(),
            index_buffer: ptr::null_mut(),
            vertex_buffer_size: 0,
            index_buffer_size: 0,
        }
    }

    pub fn device(&self) -> *mut ID3D11Device {
        self.device
    }

    pub fn devcon(&self) -> *mut ID3D11DeviceContext {
        self.devcon
    }

    fn create_render_target(
        swap_chain: *mut IDXGISwapChain,
        device: *mut ID3D11Device,
    ) -> *mut ID3D11RenderTargetView {
        let mut back_buffer: *mut ID3D11Texture2D = ptr::null_mut();
        let mut main_render_target = ptr::null_mut();
        unsafe {
            (*swap_chain).GetBuffer(
                0,
                &ID3D11Texture2D::uuidof(),
                &mut back_buffer as *mut *mut ID3D11Texture2D as *mut *mut _,
            );
            (*device).CreateRenderTargetView(
                back_buffer as *mut ID3D11Resource,
                ptr::null(),
                &mut main_render_target,
            );
            (*back_buffer).Release();
        }
        main_render_target
    }

    fn create_vertex_shader(
        device: *mut ID3D11Device,
    ) -> (
        *mut ID3D11VertexShader,
        *mut ID3D11InputLayout,
        *mut ID3D11Buffer,
    ) {
        let vertex_shader = include_str!("imgui.vs");
        let mut vertex_shader_blob = ptr::null_mut();
        let mut error_blob = ptr::null_mut();
        unsafe {
            D3DCompile(
                vertex_shader.as_ptr() as LPCVOID,
                vertex_shader.len(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                cstr!("main"),
                cstr!("vs_4_0"),
                0,
                0,
                &mut vertex_shader_blob,
                &mut error_blob,
            );
        }
        if vertex_shader_blob.is_null() {
            let error_message =
                unsafe { CStr::from_ptr((*error_blob).GetBufferPointer() as *const i8) };
            panic!("Failed to compile vertex shader: {:?}", error_message);
        }

        let mut vertex_shader = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).CreateVertexShader(
                    (*vertex_shader_blob).GetBufferPointer(),
                    (*vertex_shader_blob).GetBufferSize(),
                    ptr::null_mut(),
                    &mut vertex_shader,
                )
            },
            0
        );

        let local_layout = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: cstr!("POSITION"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(ImDrawVert => pos).get_byte_offset() as u32,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: cstr!("TEXCOORD"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(ImDrawVert => uv).get_byte_offset() as u32,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: cstr!("COLOR"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(ImDrawVert => col).get_byte_offset() as u32,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];
        let mut input_layout = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).CreateInputLayout(
                    &local_layout[0],
                    local_layout.len() as u32,
                    (*vertex_shader_blob).GetBufferPointer(),
                    (*vertex_shader_blob).GetBufferSize(),
                    &mut input_layout,
                )
            },
            0
        );

        let vertex_constant_buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: mem::size_of::<VertexConstantBuffer>() as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let mut vertex_constant_buffer = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).CreateBuffer(
                    &vertex_constant_buffer_desc,
                    ptr::null(),
                    &mut vertex_constant_buffer,
                )
            },
            0
        );

        (vertex_shader, input_layout, vertex_constant_buffer)
    }

    fn create_pixel_shader(device: *mut ID3D11Device) -> *mut ID3D11PixelShader {
        let pixel_shader = include_str!("imgui.ps");
        let mut pixel_shader_blob = ptr::null_mut();
        let mut error_blob = ptr::null_mut();
        unsafe {
            D3DCompile(
                pixel_shader.as_ptr() as LPCVOID,
                pixel_shader.len(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                cstr!("main"),
                cstr!("ps_4_0"),
                0,
                0,
                &mut pixel_shader_blob,
                &mut error_blob,
            );
        }
        if pixel_shader_blob.is_null() {
            let error_message =
                unsafe { CStr::from_ptr((*error_blob).GetBufferPointer() as *const i8) };
            panic!("Failed to compile pixel shader: {:?}", error_message);
        }

        let mut pixel_shader = ptr::null_mut();
        assert_eq!(
            unsafe {
                (*device).CreatePixelShader(
                    (*pixel_shader_blob).GetBufferPointer(),
                    (*pixel_shader_blob).GetBufferSize(),
                    ptr::null_mut(),
                    &mut pixel_shader,
                )
            },
            0
        );

        pixel_shader
    }

    fn create_font_texture(device: *mut ID3D11Device) -> *mut ID3D11ShaderResourceView {
        let mut pixels: *mut c_uchar = ptr::null_mut();
        let mut width: c_int = 0;
        let mut height: c_int = 0;
        let mut bytes_per_pixel: c_int = 0;
        unsafe {
            ImFontAtlas_GetTexDataAsRGBA32(
                (*igGetIO()).fonts,
                &mut pixels,
                &mut width,
                &mut height,
                &mut bytes_per_pixel,
            )
        };

        let desc = D3D11_TEXTURE2D_DESC {
            Width: width as u32,
            Height: height as u32,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let sub_resource = D3D11_SUBRESOURCE_DATA {
            pSysMem: pixels as *const _,
            SysMemPitch: width as u32 * 4,
            SysMemSlicePitch: 0,
        };
        let mut texture = ptr::null_mut();
        unsafe { (*device).CreateTexture2D(&desc, &sub_resource, &mut texture) };

        let mut srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
            u: unsafe { mem::zeroed() },
        };
        let mut font_texture_view = ptr::null_mut();
        unsafe {
            srv_desc.u.Texture2D_mut().MipLevels = desc.MipLevels;
            srv_desc.u.Texture2D_mut().MostDetailedMip = 0;
            (*device).CreateShaderResourceView(
                texture as *mut ID3D11Resource,
                &srv_desc,
                &mut font_texture_view,
            );
            (*texture).Release();
        }

        font_texture_view
    }

    fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            (*self.main_render_target).Release();
            (*self.swap_chain).ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, 0);
            self.main_render_target =
                D3DResources::create_render_target(self.swap_chain, self.device);
        }
    }

    fn ensure_buffer(
        device: *mut ID3D11Device,
        buffer: &mut *mut ID3D11Buffer,
        current_size: &mut usize,
        min_size: usize,
        padding: usize,
        element_size: usize,
        bind_flags: UINT,
    ) {
        if buffer.is_null() || *current_size < min_size {
            if !buffer.is_null() {
                unsafe {
                    (**buffer).Release();
                }
                *buffer = ptr::null_mut();
            }
            *current_size = min_size + padding;

            let desc = D3D11_BUFFER_DESC {
                Usage: D3D11_USAGE_DYNAMIC,
                ByteWidth: (*current_size * element_size) as u32,
                BindFlags: bind_flags,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            assert_eq!(
                unsafe { (*device).CreateBuffer(&desc, ptr::null(), buffer) },
                0
            );
        }
    }

    fn build_buffers(&mut self, draw_data: &ImDrawData, size: ImVec2) {
        unsafe {
            (*self.devcon).OMSetRenderTargets(1, &self.main_render_target, ptr::null_mut());
            (*self.devcon).ClearRenderTargetView(self.main_render_target, &[0.3, 0.3, 0.3, 1.00]);
        }

        D3DResources::ensure_buffer(
            self.device,
            &mut self.vertex_buffer,
            &mut self.vertex_buffer_size,
            draw_data.total_vtx_count as usize,
            5000,
            mem::size_of::<ImDrawVert>(),
            D3D11_BIND_VERTEX_BUFFER,
        );
        D3DResources::ensure_buffer(
            self.device,
            &mut self.index_buffer,
            &mut self.index_buffer_size,
            draw_data.total_idx_count as usize,
            10000,
            mem::size_of::<ImDrawIdx>(),
            D3D11_BIND_INDEX_BUFFER,
        );

        // copy and convert all vertices into a single contiguous buffer
        let mut vtx_resource = unsafe { mem::zeroed::<D3D11_MAPPED_SUBRESOURCE>() };
        let mut idx_resource = unsafe { mem::zeroed::<D3D11_MAPPED_SUBRESOURCE>() };
        assert_eq!(
            unsafe {
                (*self.devcon).Map(
                    self.vertex_buffer as *mut ID3D11Resource,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    &mut vtx_resource,
                )
            },
            0
        );
        assert_eq!(
            unsafe {
                (*self.devcon).Map(
                    self.index_buffer as *mut ID3D11Resource,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    &mut idx_resource,
                )
            },
            0
        );

        let mut vtx_dst = vtx_resource.pData as *mut ImDrawVert;
        let mut idx_dst = idx_resource.pData as *mut ImDrawIdx;

        for cmd_list_index in 0..draw_data.cmd_lists_count {
            let cmd_list = unsafe { &**draw_data.cmd_lists.offset(cmd_list_index as isize) };
            unsafe {
                ptr::copy_nonoverlapping(
                    cmd_list.vtx_buffer.data,
                    vtx_dst,
                    cmd_list.vtx_buffer.size as usize,
                );
                ptr::copy_nonoverlapping(
                    cmd_list.idx_buffer.data,
                    idx_dst,
                    cmd_list.idx_buffer.size as usize,
                );
                vtx_dst = vtx_dst.offset(cmd_list.vtx_buffer.size as isize);
                idx_dst = idx_dst.offset(cmd_list.idx_buffer.size as isize);
            }
        }

        unsafe {
            (*self.devcon).Unmap(self.vertex_buffer as *mut ID3D11Resource, 0);
            (*self.devcon).Unmap(self.index_buffer as *mut ID3D11Resource, 0);
        }

        // setup orthographic projection matrix into constant buffer
        let mut constant_buffer_resource = unsafe { mem::zeroed::<D3D11_MAPPED_SUBRESOURCE>() };
        assert_eq!(
            unsafe {
                (*self.devcon).Map(
                    self.vertex_constant_buffer as *mut ID3D11Resource,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    &mut constant_buffer_resource,
                )
            },
            0
        );
        let constant_buffer_dst = constant_buffer_resource.pData as *mut VertexConstantBuffer;

        unsafe {
            (*constant_buffer_dst).mvp = [
                [2. / size.x, 0., 0., 0.],
                [0., -2. / size.y, 0., 0.],
                [0., 0., -1., 0.],
                [-1., 1., 0., 1.],
            ];
            (*self.devcon).Unmap(self.vertex_constant_buffer as *mut ID3D11Resource, 0);
        }
    }

    fn render_frame(&self, draw_data: &ImDrawData, size: ImVec2, scale: ImVec2) {
        // backup the state
        let mut backup_state: BackupState = unsafe { mem::zeroed() };
        backup_state.scissor_rects_count = D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE;
        backup_state.viewports_count = D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE;
        backup_state.ps_instances_count = 256;
        backup_state.vs_instances_count = 256;
        unsafe {
            (*self.devcon).RSGetScissorRects(
                &mut backup_state.scissor_rects_count,
                &mut backup_state.scissor_rects[0],
            );
            (*self.devcon).RSGetViewports(
                &mut backup_state.viewports_count,
                &mut backup_state.viewports[0],
            );
            (*self.devcon).RSGetState(&mut backup_state.raster_state);
            (*self.devcon).OMGetBlendState(
                &mut backup_state.blend_state,
                &mut backup_state.blend_factor,
                &mut backup_state.sample_mask,
            );
            (*self.devcon).OMGetDepthStencilState(
                &mut backup_state.depth_stencil_state,
                &mut backup_state.stencil_ref,
            );
            (*self.devcon).PSGetShaderResources(0, 1, &mut backup_state.ps_shader_resource);
            (*self.devcon).PSGetSamplers(0, 1, &mut backup_state.ps_sampler);
            (*self.devcon).PSGetShader(
                &mut backup_state.ps,
                &mut backup_state.ps_instances[0],
                &mut backup_state.ps_instances_count,
            );
            (*self.devcon).VSGetShader(
                &mut backup_state.vs,
                &mut backup_state.vs_instances[0],
                &mut backup_state.vs_instances_count,
            );
            (*self.devcon).VSGetConstantBuffers(0, 1, &mut backup_state.vs_constant_buffer);
            (*self.devcon).IAGetPrimitiveTopology(&mut backup_state.primitive_topology);
            (*self.devcon).IAGetIndexBuffer(
                &mut backup_state.index_buffer,
                &mut backup_state.index_buffer_format,
                &mut backup_state.index_buffer_offset,
            );
            (*self.devcon).IAGetVertexBuffers(
                0,
                1,
                &mut backup_state.vertex_buffer,
                &mut backup_state.vertex_buffer_stride,
                &mut backup_state.vertex_buffer_offset,
            );
            (*self.devcon).IAGetInputLayout(&mut backup_state.input_layout);
        }

        let vp = D3D11_VIEWPORT {
            Width: size.x,
            Height: size.y,
            MinDepth: 0.,
            MaxDepth: 1.,
            TopLeftX: 0.,
            TopLeftY: 0.,
        };
        unsafe {
            (*self.devcon).RSSetViewports(1, &vp);
        }

        // bind shader and vertex buffers
        let stride = mem::size_of::<ImDrawVert>() as u32;
        let offset = 0;
        unsafe {
            (*self.devcon).IASetInputLayout(self.input_layout);
            (*self.devcon).IASetVertexBuffers(0, 1, &self.vertex_buffer, &stride, &offset);
            (*self.devcon).IASetIndexBuffer(
                self.index_buffer,
                if mem::size_of::<ImDrawIdx>() == 2 {
                    DXGI_FORMAT_R16_UINT
                } else {
                    DXGI_FORMAT_R32_UINT
                },
                0,
            );
            (*self.devcon).IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            (*self.devcon).VSSetShader(self.vertex_shader, ptr::null(), 0);
            (*self.devcon).VSSetConstantBuffers(0, 1, &self.vertex_constant_buffer);
            (*self.devcon).PSSetShader(self.pixel_shader, ptr::null(), 0);
            (*self.devcon).PSSetSamplers(0, 1, &self.sampler);
        }

        // setup render state
        unsafe {
            (*self.devcon).OMSetBlendState(self.blend_state, &[0., 0., 0., 0.], 0xffff_ffff);
            (*self.devcon).OMSetDepthStencilState(self.depth_stencil_state, 0);
            (*self.devcon).RSSetState(self.rasterizer_state);
        }

        // render command lists
        let mut vtx_offset = 0;
        let mut idx_offset = 0;
        for draw_list_index in 0..draw_data.cmd_lists_count {
            let draw_list = unsafe { &**draw_data.cmd_lists.offset(draw_list_index as isize) };

            for cmd_index in 0..draw_list.cmd_buffer.size {
                let cmd = unsafe { &*draw_list.cmd_buffer.data.offset(cmd_index as isize) };

                if let Some(user_callback) = cmd.user_callback {
                    user_callback(draw_list, cmd);
                } else {
                    let clip_rect = D3D11_RECT {
                        left: (cmd.clip_rect.x * scale.x) as i32,
                        top: (cmd.clip_rect.y * scale.y) as i32,
                        right: (cmd.clip_rect.z * scale.x) as i32,
                        bottom: (cmd.clip_rect.w * scale.y) as i32,
                    };
                    unsafe {
                        (*self.devcon).RSSetScissorRects(1, &clip_rect);
                        (*self.devcon).PSSetShaderResources(
                            0,
                            1,
                            &cmd.texture_id as *const *mut c_void as *const *mut _,
                        );
                        (*self.devcon).DrawIndexed(cmd.elem_count, idx_offset, vtx_offset);
                    }

                    idx_offset += cmd.elem_count;
                }
            }

            vtx_offset += draw_list.vtx_buffer.size as i32;
        }

        // restore the backed up state
        unsafe {
            (*self.devcon).RSSetScissorRects(
                backup_state.scissor_rects_count,
                &backup_state.scissor_rects[0],
            );
            (*self.devcon).RSSetViewports(backup_state.viewports_count, &backup_state.viewports[0]);
            (*self.devcon).RSSetState(backup_state.raster_state);
            if !backup_state.raster_state.is_null() {
                (*backup_state.raster_state).Release();
            }
            (*self.devcon).OMSetBlendState(
                backup_state.blend_state,
                &backup_state.blend_factor,
                backup_state.sample_mask,
            );
            if !backup_state.blend_state.is_null() {
                (*backup_state.blend_state).Release();
            }
            (*self.devcon)
                .OMSetDepthStencilState(backup_state.depth_stencil_state, backup_state.stencil_ref);
            if !backup_state.depth_stencil_state.is_null() {
                (*backup_state.depth_stencil_state).Release();
            }
            (*self.devcon).PSSetShaderResources(0, 1, &backup_state.ps_shader_resource);
            if !backup_state.ps_shader_resource.is_null() {
                (*backup_state.ps_shader_resource).Release();
            }
            (*self.devcon).PSSetSamplers(0, 1, &backup_state.ps_sampler);
            if !backup_state.ps_sampler.is_null() {
                (*backup_state.ps_sampler).Release();
            }
            (*self.devcon).PSSetShader(
                backup_state.ps,
                &backup_state.ps_instances[0],
                backup_state.ps_instances_count,
            );
            if !backup_state.ps.is_null() {
                (*backup_state.ps).Release();
            }
            for i in 0..backup_state.ps_instances_count {
                if !backup_state.ps_instances[i as usize].is_null() {
                    (*backup_state.ps_instances[i as usize]).Release();
                }
            }
            (*self.devcon).VSSetShader(
                backup_state.vs,
                &backup_state.vs_instances[0],
                backup_state.vs_instances_count,
            );
            if !backup_state.vs.is_null() {
                (*backup_state.vs).Release();
            }
            (*self.devcon).VSSetConstantBuffers(0, 1, &backup_state.vs_constant_buffer);
            if !backup_state.vs_constant_buffer.is_null() {
                (*backup_state.vs_constant_buffer).Release();
            }
            for i in 0..backup_state.vs_instances_count {
                if !backup_state.vs_instances[i as usize].is_null() {
                    (*backup_state.vs_instances[i as usize]).Release();
                }
            }
            (*self.devcon).IASetPrimitiveTopology(backup_state.primitive_topology);
            (*self.devcon).IASetIndexBuffer(
                backup_state.index_buffer,
                backup_state.index_buffer_format,
                backup_state.index_buffer_offset,
            );
            if !backup_state.index_buffer.is_null() {
                (*backup_state.index_buffer).Release();
            }
            (*self.devcon).IASetVertexBuffers(
                0,
                1,
                &backup_state.vertex_buffer,
                &backup_state.vertex_buffer_stride,
                &backup_state.vertex_buffer_offset,
            );
            if !backup_state.vertex_buffer.is_null() {
                (*backup_state.vertex_buffer).Release();
            }
            (*self.devcon).IASetInputLayout(backup_state.input_layout);
            if !backup_state.input_layout.is_null() {
                (*backup_state.input_layout).Release();
            }
        }
    }

    fn swap_buffers(&self) {
        unsafe {
            (*self.swap_chain).Present(1, 0);
        }
    }
}

impl Drop for D3DResources {
    fn drop(&mut self) {
        unsafe {
            (*self.device).Release();
            (*self.devcon).Release();
            (*self.main_render_target).Release();
            (*self.swap_chain).Release();
            (*self.input_layout).Release();
            (*self.vertex_shader).Release();
            (*self.pixel_shader).Release();
            (*self.font_texture_view).Release();
            (*self.sampler).Release();
            (*self.blend_state).Release();
            (*self.depth_stencil_state).Release();
            (*self.rasterizer_state).Release();
            (*self.vertex_constant_buffer).Release();

            if !self.vertex_buffer.is_null() {
                (*self.vertex_buffer).Release();
            }
            if !self.index_buffer.is_null() {
                (*self.index_buffer).Release();
            }
        }
    }
}

pub struct ImGuiWindow {
    pub window: HWND,
    pub resources: D3DResources,

    ticks_per_sec: u64,
    last_time: u64,
    last_mouse_cursor: ImGuiMouseCursor,
    last_window_size: ImVec2,
}

impl ImGuiWindow {
    pub fn new(window: HWND) -> Self {
        unsafe {
            igCreateContext(ptr::null_mut());
        }

        unsafe {
            let io = igGetIO();

            (*io).backend_flags |= ImGuiBackendFlags::HasMouseCursors;
            (*io).backend_flags |= ImGuiBackendFlags::HasSetMousePos;
            (*io).backend_flags |= ImGuiBackendFlags::PlatformHasViewports;
            (*io).backend_flags |= ImGuiBackendFlags::HasMouseHoveredViewports;
            (*io).backend_platform_name = cstr!("re19");

            (*io).config_flags |= ImGuiConfigFlags::DockingEnable;

            // setup keyboard mapping
            (*io).key_map[ImGuiKey::Tab as usize] = VK_TAB;
            (*io).key_map[ImGuiKey::LeftArrow as usize] = VK_LEFT;
            (*io).key_map[ImGuiKey::RightArrow as usize] = VK_RIGHT;
            (*io).key_map[ImGuiKey::UpArrow as usize] = VK_UP;
            (*io).key_map[ImGuiKey::DownArrow as usize] = VK_DOWN;
            (*io).key_map[ImGuiKey::PageUp as usize] = VK_PRIOR;
            (*io).key_map[ImGuiKey::PageDown as usize] = VK_NEXT;
            (*io).key_map[ImGuiKey::Home as usize] = VK_HOME;
            (*io).key_map[ImGuiKey::End as usize] = VK_END;
            (*io).key_map[ImGuiKey::Insert as usize] = VK_INSERT;
            (*io).key_map[ImGuiKey::Delete as usize] = VK_DELETE;
            (*io).key_map[ImGuiKey::Backspace as usize] = VK_BACK;
            (*io).key_map[ImGuiKey::Space as usize] = VK_SPACE;
            (*io).key_map[ImGuiKey::Enter as usize] = VK_RETURN;
            (*io).key_map[ImGuiKey::Escape as usize] = VK_ESCAPE;
            (*io).key_map[ImGuiKey::A as usize] = 'A' as i32;
            (*io).key_map[ImGuiKey::C as usize] = 'C' as i32;
            (*io).key_map[ImGuiKey::V as usize] = 'V' as i32;
            (*io).key_map[ImGuiKey::X as usize] = 'X' as i32;
            (*io).key_map[ImGuiKey::Y as usize] = 'Y' as i32;
            (*io).key_map[ImGuiKey::Z as usize] = 'Z' as i32;
        }

        unsafe {
            let main_viewport = igGetMainViewport();
            (*main_viewport).platform_handle = window as *mut _;
        }

        let mut ticks_per_sec = 0;
        unsafe { QueryPerformanceFrequency(&mut ticks_per_sec as *mut u64 as *mut _) };
        let mut last_time = 0;
        unsafe { QueryPerformanceCounter(&mut last_time as *mut u64 as *mut _) };

        let resources = D3DResources::new(window);

        ImGuiWindow {
            window,
            resources,
            ticks_per_sec,
            last_time,
            last_mouse_cursor: ImGuiMouseCursor::None,
            last_window_size: ImGuiWindow::get_window_size(window),
        }
    }

    fn try_take_capture(window: HWND) {
        unsafe {
            if !igIsAnyMouseDown() && GetCapture().is_null() {
                SetCapture(window);
            }
        }
    }

    fn try_lose_capture(window: HWND) {
        unsafe {
            if !igIsAnyMouseDown() && GetCapture() == window {
                ReleaseCapture();
            }
        }
    }

    unsafe fn shared_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if igGetCurrentContext().is_null() {
            return 0;
        }

        let io = igGetIO();

        match msg {
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
                ImGuiWindow::try_take_capture(hwnd);
                (*io).mouse_down[0] = true;
                0
            }
            WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
                ImGuiWindow::try_take_capture(hwnd);
                (*io).mouse_down[1] = true;
                0
            }
            WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
                ImGuiWindow::try_take_capture(hwnd);
                (*io).mouse_down[2] = true;
                0
            }

            WM_LBUTTONUP => {
                (*io).mouse_down[0] = false;
                ImGuiWindow::try_lose_capture(hwnd);
                0
            }
            WM_RBUTTONUP => {
                (*io).mouse_down[1] = false;
                ImGuiWindow::try_lose_capture(hwnd);
                0
            }
            WM_MBUTTONUP => {
                (*io).mouse_down[2] = false;
                ImGuiWindow::try_lose_capture(hwnd);
                0
            }

            WM_MOUSEWHEEL => {
                (*io).mouse_wheel += GET_WHEEL_DELTA_WPARAM(wparam) as f32 / WHEEL_DELTA as f32;
                0
            }
            WM_MOUSEHWHEEL => {
                (*io).mouse_wheel_h += GET_WHEEL_DELTA_WPARAM(wparam) as f32 / WHEEL_DELTA as f32;
                0
            }

            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam < 256 {
                    (*io).keys_down[wparam] = true;
                }
                0
            }
            WM_KEYUP | WM_SYSKEYUP => {
                if wparam < 256 {
                    (*io).keys_down[wparam] = false;
                }
                0
            }

            WM_CHAR => {
                if wparam > 0 && wparam < 0x10000 {
                    ImGuiIO_AddInputCharacter(io, wparam as c_ushort);
                }
                0
            }

            WM_SETCURSOR => {
                if LOWORD(lparam as u32) == HTCLIENT as u16 && ImGuiWindow::update_mouse_cursor() {
                    1
                } else {
                    0
                }
            }

            _ => 0,
        }
    }

    pub unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if ImGuiWindow::shared_wnd_proc(hwnd, msg, wparam, lparam) != 0 {
            return 1;
        }

        if msg == WM_CLOSE {
            PostQuitMessage(0);
            0
        } else {
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
    }

    fn update_mouse_cursor() -> bool {
        let io = unsafe { igGetIO() };
        if unsafe { (*io).config_flags }.contains(ImGuiConfigFlags::NoMouseCursorChange) {
            return false;
        }

        let current_cursor = unsafe { igGetMouseCursor() };
        if current_cursor == ImGuiMouseCursor::None || unsafe { (*io).mouse_draw_cursor } {
            unsafe { SetCursor(ptr::null_mut()) };
        } else {
            // show OS mouse cursor
            let display_cursor = match current_cursor {
                ImGuiMouseCursor::None => panic!(),
                ImGuiMouseCursor::Arrow => IDC_ARROW,
                ImGuiMouseCursor::TextInput => IDC_IBEAM,
                ImGuiMouseCursor::ResizeAll => IDC_SIZEALL,
                ImGuiMouseCursor::ResizeEW => IDC_SIZEWE,
                ImGuiMouseCursor::ResizeNS => IDC_SIZENS,
                ImGuiMouseCursor::ResizeNESW => IDC_SIZENESW,
                ImGuiMouseCursor::ResizeNWSE => IDC_SIZENWSE,
                ImGuiMouseCursor::Hand => IDC_HAND,
            };
            unsafe {
                SetCursor(LoadCursorA(ptr::null_mut(), display_cursor as *const _));
            }
        }

        true
    }

    fn update_mouse_pos(&self) {
        let io = unsafe { igGetIO() };

        if unsafe { (*io).want_set_mouse_pos } {
            let mut pos = POINT {
                x: unsafe { (*io).mouse_pos.x } as LONG,
                y: unsafe { (*io).mouse_pos.y } as LONG,
            };
            unsafe {
                ClientToScreen(self.window, &mut pos);
                SetCursorPos(pos.x, pos.y);
            }
        }

        unsafe {
            (*io).mouse_pos = ImVec2 {
                x: f32::MIN,
                y: f32::MIN,
            }
        }
        let mut pos = POINT { x: 0, y: 0 };
        let active_window = unsafe { GetForegroundWindow() };
        if !active_window.is_null()
            && (active_window == self.window || unsafe { IsChild(active_window, self.window) } != 0)
        {
            if unsafe { GetCursorPos(&mut pos) != 0 }
                && unsafe { ScreenToClient(self.window, &mut pos) != 0 }
            {
                unsafe {
                    (*io).mouse_pos = ImVec2 {
                        x: pos.x as f32,
                        y: pos.y as f32,
                    }
                }
            }
        }
    }

    fn get_window_size(window: HWND) -> ImVec2 {
        let mut rect = unsafe { mem::zeroed() };
        unsafe { GetClientRect(window, &mut rect) };
        ImVec2 {
            x: (rect.right - rect.left) as f32,
            y: (rect.bottom - rect.top) as f32,
        }
    }

    pub fn start_frame(&mut self) {
        let io = unsafe { igGetIO() };

        let display_size = ImGuiWindow::get_window_size(self.window);

        if display_size != self.last_window_size {
            self.resources
                .resize(display_size.x as u32, display_size.y as u32);
            self.last_window_size = display_size;
        }

        unsafe { (*io).display_size = display_size };

        // update passed time
        let mut current_time = 0;
        unsafe { QueryPerformanceCounter(&mut current_time as *mut u64 as *mut _) };
        unsafe {
            (*io).delta_time = (current_time - self.last_time) as f32 / self.ticks_per_sec as f32
        };
        self.last_time = current_time;

        // read keyboard modifiers
        unsafe {
            (*io).key_ctrl = (GetKeyState(VK_CONTROL) as u16 & 0x8000) != 0;
            (*io).key_shift = (GetKeyState(VK_SHIFT) as u16 & 0x8000) != 0;
            (*io).key_alt = (GetKeyState(VK_MENU) as u16 & 0x8000) != 0;
            (*io).key_super = false;
        }

        // update OS mouse position
        self.update_mouse_pos();

        // update OS mouse cursor
        let mouse_cursor = if unsafe { (*io).mouse_draw_cursor } {
            ImGuiMouseCursor::None
        } else {
            unsafe { igGetMouseCursor() }
        };
        if self.last_mouse_cursor != mouse_cursor {
            self.last_mouse_cursor = mouse_cursor;
            ImGuiWindow::update_mouse_cursor();
        }

        unsafe {
            igNewFrame();
        }
    }

    pub fn end_frame(&mut self) {
        unsafe {
            igRender();
        };
        let draw_data = unsafe { &*igGetDrawData() };
        self.resources
            .build_buffers(draw_data, self.last_window_size);
        self.resources
            .render_frame(draw_data, self.last_window_size, ImVec2 { x: 1., y: 1. });
        self.resources.swap_buffers();
    }

    pub fn poll_events(&self) -> bool {
        let mut msg = unsafe { mem::uninitialized() };
        while unsafe { PeekMessageA(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
            if msg.message == WM_QUIT {
                return false;
            }

            unsafe {
                TranslateMessage(&msg);
            }

            unsafe {
                DispatchMessageA(&msg);
            }
        }
        true
    }
}

/*struct ViewportData {
    hwnd: HWND,
    hwnd_owned: bool,
    dw_style: DWORD,
    dw_ex_style: DWORD,
}

impl ViewportData {
    fn get_style_from_viewport_flags(flags: ImGuiViewportFlags) -> (DWORD, DWORD) {
        let style = if flags.contains(ImGuiViewportFlags::NoDecoration) {
            WS_POPUP
        } else {
            WS_OVERLAPPEDWINDOW
        };

        let mut ex_style = if flags.contains(ImGuiViewportFlags::NoTaskBarIcon) {
            WS_EX_TOOLWINDOW
        } else {
            WS_EX_APPWINDOW
        };
        if flags.contains(ImGuiViewportFlags::TopMost) {
            ex_style |= WS_EX_TOPMOST;
        }

        (style, ex_style)
    }

    fn create_window(viewport: &mut ImGuiViewport) {
        let data_ptr = Box::into_raw(Box::new(ViewportData {
            hwnd: ptr::null_mut(),
            hwnd_owned: false,
            dw_style: 0,
            dw_ex_style: 0,
        }));
        let data_ref = unsafe { &mut *data_ptr };

        viewport.platform_user_data = data_ptr as *mut _;

        // select style and parent window
        let (dw_style, dw_ex_style) = ViewportData::get_style_from_viewport_flags(viewport.flags);
        data_ref.dw_style = dw_style;
        data_ref.dw_ex_style = dw_ex_style;

        let mut parent_window: HWND = ptr::null_mut();
        if viewport.parent_viewport_id != 0 {
            let parent_viewport = unsafe { igFindViewportById(viewport.parent_viewport_id) };
            if !parent_viewport.is_null() {
                parent_window = unsafe { (*parent_viewport).platform_handle } as *mut _;
            }
        }

        // create the window
        let mut rect = RECT {
            left: viewport.pos.x as i32,
            top: viewport.pos.y as i32,
            right: (viewport.pos.x + viewport.size.x) as i32,
            bottom: (viewport.pos.y + viewport.size.y) as i32,
        };
        unsafe {
            AdjustWindowRectEx(&mut rect, data_ref.dw_style, 0, data_ref.dw_ex_style)
        };
        data_ref.hwnd = unsafe {
            CreateWindowExA(
                data_ref.dw_ex_style,
                cstr!("ImGui Platform"),
                cstr!("Untitled"),
                data_ref.dw_style,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                parent_window,
                ptr::null_mut(),
                GetModuleHandleA(ptr::null_mut()),
                ptr::null_mut()
            )
        };
        data_ref.hwnd_owned = true;
        viewport.platform_request_resize = false;
        viewport.platform_handle = data_ref.hwnd as *mut _;
    }

    fn destroy_window(viewport: &mut ImGuiViewport, main_window: HWND) {
        let data_ptr = viewport.platform_user_data as *mut ViewportData;
        if !data_ptr.is_null() {
            let data_ref = unsafe { &mut *data_ptr };

            if unsafe { GetCapture() } == data_ref.hwnd {
                // transfer capture back to the main window
                unsafe {
                    ReleaseCapture();
                    SetCapture(main_window);
                }
            }

            if !data_ref.hwnd.is_null() && data_ref.hwnd_owned {
                unsafe { DestroyWindow(data_ref.hwnd) };
            }

            // dispose of the data
            mem::drop(Box::from_raw(data_ptr));
        }

        viewport.platform_user_data = ptr::null_mut();
        viewport.platform_handle = ptr::null_mut();
    }

    fn show_window(viewport: &mut ImGuiViewport) {
        let data = let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        if viewport.flags.contains(ImGuiViewportFlags::NoFocusOnAppearing) {
            unsafe { ShowWindow(data.hwnd, SW_SHOWNA); }
        } else {
            unsafe { ShowWindow(data.hwnd, SW_SHOW) };
        }
    }

    fn update_window(viewport: &mut ImGuiViewport) {
        let data = let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        let (new_style, new_ex_style) = ViewportData::get_style_from_viewport_flags(viewport.flags);

        // only reapply flags that have changed
        if data.dw_style != new_style || data.dw_ex_style != new_ex_style {
            data.dw_style = new_style;
            data.dw_ex_style = new_ex_style;

            unsafe {
                SetWindowLongA(data.hwnd, GWL_STYLE, data.dw_style);
                SetWindowLongA(data.hwnd, GWL_EXSTYLE, data.dw_ex_style);
            }

            let mut rect = RECT {
                left: viewport.pos.x,
                top: viewport.pos.y,
                right: viewport.pos.x + viewport.size.x,
                bottom: viewport.pos.y + viewport.size.y,
            };

            unsafe {
                AdjustWindowRectEx(&mut rect, data.dw_style, 0, data.dw_ex_style);
                SetWindowPos(data.hwnd, ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
                ShowWindow(data.hwnd, SW_SHOWNA);
            }
            viewport.platform_request_move = true;
            viewport.platform_request_resize = true;
        }
    }

    fn get_window_pos(viewport: &mut ImGuiViewport) -> ImVec2 {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        let mut pos = POINT { x: 0, y: 0 };
        unsafe { ClientToScreen(dat.hwnd, &mut pos) };
        ImVec2 {
            x: pos.x as f32,
            y: pos.y as f32,
        }
    }

    fn set_window_pos(viewport: &mut ImGuiViewport, pos: ImVec2) {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        let mut rect = RECT {
            left: pos.x,
            top: pos.y,
            right: pos.x,
            bottom: pos.y
        };
        unsafe {
            AdjustWindowRectEx(&mut rect, data.dw_style, 0, data.dw_ex_style);
            SetWindowPos(data.hwnd, ptr::null_mut(), rect.left, rect.top, 0, 0, SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }

    fn get_window_size(viewport: &mut ImGuiViewport) -> ImVec2 {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        let mut rect = unsafe { mem::zeroed() };
        unsafe {
            GetClientRect(data.hwnd, &mut rect);
        }

        ImVec2 {
            x: (rect.right - rect.left) as f32,
            y: (rect.bottom - rect.top) as f32,
        }
    }

    fn set_window_size(viewport: &mut ImGuiViewport, size: ImVec2) {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: size.x,
            bottom: size.y,
        };

        unsafe {
            AdjustWindowRectEx(&mut rect, data.dw_style, 0, data.dw_ex_style);
            SetWindowPos(data.hwnd, ptr::null_mut(), 0, 0, rect.right - rect.left, rect.bottom - rect.top, SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE);
        }
    }

    fn set_window_focus(viewport: &mut ImGuiViewport) {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        unsafe {
            BringWindowToTop(data.hwnd);
            SetForegroundWindow(data.hwnd);
            SetFocus(data.hwnd);
        }
    }

    fn get_window_focus(viewport: &mut ImGuiViewport) -> bool {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        unsafe {
            GetForegroundWindow() == data.hwnd
        }
    }

    fn get_window_minimized(viewport: &mut ImGuiViewport) -> bool {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };
        unsafe {
            IsIconic(data.hwnd) != 0
        }
    }

    fn set_window_title(viewport: &mut ImGuiViewport, title: *const c_char) {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };

        unsafe {
            let n = MultiByteToWideChar(CP_UTF8, 0, title, -1, ptr::null_mut(), 0);
            let title_w = vec![0; n];
            MultiByteToWideChar(CP_UTF*, 0, title, -1, &title_w[0], n);
            SetWindowTextW(data.hwnd, &title_w[0]);
        }
    }

    fn set_window_alpha(viewport: &mut ImGuiViewport, alpha: f32) {
        let data = unsafe { &mut *(viewport.platform_user_data as *mut ViewportData) };

        if alpha < 1. {
            unsafe {
                let style = GetWindowLongA(data.hwnd, GWL_EXSTYLE) | WS_EX_LAYERED;
                SetWindowLongA(data.hwnd, GWL_EXSTYLE, style);
                SetLayeredWindowAttributes(data.hwnd, 0, 255 * alpha, LWA_ALPHA);
            }
        } else {
            unsafe {
                let style = GetWindowLongA(data.hwnd, GWL_EXSTYLE) & ~WS_EX_LAYERED;
                SetWindowLongA(data.hwnd, GWL_EXSTYLE, style);
            }
        }
    }

    fn get_window_dpi_scale(viewport: &mut ImGuiViewport) -> f32 {
        1.
    }

    unsafe extern "system" fn platform_window_proc_handler(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if ImGuiWindow::shared_wnd_proc(hwnd, msg, wparam, lparam) != 0 {
            return 1;
        }

        let viewport_ptr = igFindViewportByPlatformHandle(hwnd as *mut _);
        if !viewport_ptr.is_null() {
            let viewport = &mut *viewport_ptr;

            match msg {
                WM_CLOSE => {
                    viewport.platform_request_close = true;
                    return 0;
                }
                WM_MOVE => {
                    viewport.platform_request_move = true;
                }
                WM_SIZE => {
                    viewport.platform_request_resize = true;
                }
                WM_NCHITTEST => {
                    if viewport.flags.contains(ImGuiViewportFlags::NoInputs) {
                        return HTTRANSPARENT;
                    }
                }
                _ => {}
            }
        }

        DefWindowProcA(hwnd, msg, wparma, lparma)
    }

    unsafe extern "system" fn update_monitors_enum(monitor: HMONITOR, _hdc: HDC, _lprect: LPRECT, _lparam: LPARAM) -> BOOL {
        let mut info = unsafe { mem::zeroed::<MONITORINFO>() };
        info.cbSize = mem::size_of::<MONITORINFO>();
        if !GetMonitorInfoA(monitor, &mut info) {
            return 1;
        }

        let imgui_monitor = ImGuiPlatformMonitor {
            main_pos: ImVec2::new(info.rcMonitor.left as f32, info.rcMonitor.top as f32),
            main_size: ImVec2::new((info.rcMonitor.right - info.rcMonitor.left) as f32, (info.rcMonitor.bottom - info.rcMonitor.top) as f32),
            work_pos: ImVec2::new(info.rcWork.left as f32, info.rcWork.right as f32),
            work_size: ImVec2::new((info.rcWork.right - info.rcWork.left) as f32, (info.rcWork.bottom - info.rcWork.top) as f32),
            dpi_scale: 1., // todo?
        };
        let io = igGetPlatformIO();
        if info.dwFlags & MONITORINFOF_PRIMARY != 0 {
            ImVector_ImGuiPlatformMonitor_push_front(&mut (*io).monitors, &imgui_monitor);
        } else {
            ImVector_ImGuiPlatformMonitor_push_back(&mut (*io).monitors, &imgui_monitor);
        }

        return 1;
    }

    fn update_monitors() {
        unsafe {
            ImVector_ImGuiPlatformMonitor_resize(&mut (*igGetPlatformIO()).monitors, 0);
            EnumDisplayMonitors(ptr::null_mut(), ptr::null_mut(), ViewportData::update_monitors_enum, ptr::null_mut());
            // todo: set wantUpdateMonitors?
        }
    }
}*/
