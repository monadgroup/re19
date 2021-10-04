use crate::camera::CameraBuffer;
use crate::creation_context::CreationContext;
use crate::frame_context::{FrameContext, FrameDataBuffer};
use crate::math::{Matrix4, Vector2, Vector3, Vector4};
use crate::texture::{DepthStencil, RenderTarget2D, ShaderResource2D};
use crate::viewport::Viewport;
use core::ptr;
use winapi::um::d3d11::D3D11_VIEWPORT;

pub const SHADOW_MAP_TEX_SIZE: (u32, u32) = (4096, 4096);

pub struct ShadowMapState {
    current_camera: CameraBuffer,
    current_frame: FrameDataBuffer,
}

pub struct ShadowMapRenderer {
    shadow_map: DepthStencil,
}

impl ShadowMapRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        ShadowMapRenderer {
            shadow_map: DepthStencil::new(
                context.device,
                Viewport {
                    width: SHADOW_MAP_TEX_SIZE.0,
                    height: SHADOW_MAP_TEX_SIZE.1,
                },
            ),
        }
    }

    pub fn shadow_map(&self) -> &dyn ShaderResource2D {
        &self.shadow_map
    }

    pub fn render_start(
        &mut self,
        context: &mut FrameContext,
        pos: Vector3,
        x_range: Vector2,
        y_range: Vector2,
        z_range: Vector2,
    ) -> (&mut DepthStencil, D3D11_VIEWPORT, Matrix4, ShadowMapState) {
        let view_matrix = Matrix4::translate(pos)
            * context
                .common
                .light_data
                .world_light_rotation
                .as_matrix()
                .inverted();
        let proj_matrix = Matrix4::project_orthographic(x_range, y_range, z_range);
        let view_proj_matrix = proj_matrix * view_matrix;

        let current_camera = context.common.camera_data;
        context.common.camera_data = CameraBuffer {
            cam_position: pos,
            cam_fov_radians: 0.,
            cam_direction: context.common.light_data.world_light_direction,
            z_range: Vector4 {
                x: z_range.x - pos.z,
                y: z_range.y - pos.z,
                z: 0.,
                w: 0.,
            },
            view_matrix,
            proj_matrix,
            view_proj_matrix,
            last_matrix: view_proj_matrix,
            inv_view_matrix: view_matrix.inverted(),
            inv_proj_matrix: proj_matrix.inverted(),
            norm_view_matrix: view_matrix.transform_normal(),
        };
        context
            .common
            .camera_buffer
            .upload(context.devcon, context.common.camera_data);

        let current_frame = context.common.frame_data;
        context.common.frame_data = FrameDataBuffer {
            viewport: Vector2 {
                x: SHADOW_MAP_TEX_SIZE.0 as f32,
                y: SHADOW_MAP_TEX_SIZE.1 as f32,
            },
            seed: current_frame.seed,
        };
        context
            .common
            .frame_data_buffer
            .upload(context.devcon, context.common.frame_data);

        self.shadow_map.clear(context.devcon);
        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.,
            TopLeftY: 0.,
            Width: SHADOW_MAP_TEX_SIZE.0 as f32,
            Height: SHADOW_MAP_TEX_SIZE.1 as f32,
            MinDepth: 0.,
            MaxDepth: 1.,
        };
        (
            &mut self.shadow_map,
            viewport,
            view_proj_matrix,
            ShadowMapState {
                current_camera,
                current_frame,
            },
        )
    }

    pub fn render_end(&mut self, context: &mut FrameContext, state: ShadowMapState) {
        context.common.camera_data = state.current_camera;
        context
            .common
            .camera_buffer
            .upload(context.devcon, state.current_camera);

        context.common.frame_data = state.current_frame;
        context
            .common
            .frame_data_buffer
            .upload(context.devcon, state.current_frame);
    }
}
