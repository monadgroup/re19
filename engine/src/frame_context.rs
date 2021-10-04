use crate::animation::clip::GeneratorClipMap;
use crate::buffer::{Buffer, InitialData};
use crate::camera::CameraBuffer;
use crate::creation_context::CreationContext;
use crate::math::{Quaternion, Vector2, Vector4};
use crate::object::ObjectBuffer;
use crate::object::QuadObject;
use crate::resources::perf_table::PerfTable;
use crate::resources::shader_manager::ShaderManager;
use crate::viewport::Viewport;
use core::mem;
use winapi::um::d3d11::ID3D11DeviceContext;
use winapi::um::d3d11::D3D11_BIND_CONSTANT_BUFFER;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrameDataBuffer {
    pub viewport: Vector2,
    pub seed: f32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LightBuffer {
    pub world_light_direction: Vector4,
    pub world_light_color: Vector4,
    pub world_light_ambient: f32,
    pub world_light_rotation: Quaternion, // todo: don't include this in the buffer!
}

pub struct CommonData {
    pub object_buffer: Buffer<ObjectBuffer>,
    pub camera_buffer: Buffer<CameraBuffer>,
    pub camera_data: CameraBuffer,
    pub frame_data_buffer: Buffer<FrameDataBuffer>,
    pub frame_data: FrameDataBuffer,
    pub light_buffer: Buffer<LightBuffer>,
    pub light_data: LightBuffer,
    pub screen_quad: QuadObject,
}

impl CommonData {
    pub fn new(context: &mut CreationContext) -> Self {
        CommonData {
            object_buffer: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            camera_buffer: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            camera_data: unsafe { mem::zeroed() },
            frame_data_buffer: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            frame_data: unsafe { mem::zeroed() },
            light_buffer: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            light_data: unsafe { mem::zeroed() },
            screen_quad: QuadObject::new(
                context,
                Vector4 {
                    x: -1.,
                    y: -1.,
                    z: 1.,
                    w: 1.,
                },
            ),
        }
    }
}

pub struct FrameContext<'frame, 'perf, 'shader> {
    pub devcon: *mut ID3D11DeviceContext,
    pub delta_seconds: f32,
    pub viewport: Viewport,
    pub shader_manager: &'frame ShaderManager<'shader>,
    pub clip_map: &'frame mut GeneratorClipMap,
    pub common: &'frame mut CommonData,
    pub perf: &'frame mut PerfTable<'perf>,
}
