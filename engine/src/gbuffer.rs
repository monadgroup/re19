use crate::math::RgbaColor;
use crate::texture::{DepthStencil, RenderTarget2D, ShaderResource2D, Texture2D};
use crate::unordered_view::UnorderedView;
use crate::viewport::Viewport;
use core::mem;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT;
use winapi::um::d3d11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11ShaderResourceView,
    D3D11_BIND_UNORDERED_ACCESS,
};

pub struct GBuffer {
    lit_scene_read: Texture2D,
    lit_scene_write: Texture2D,

    normal_map: Texture2D,
    world_pos_map_read: Texture2D,
    world_pos_map_read_uav: UnorderedView,
    world_pos_map_write: Texture2D,
    world_pos_map_write_uav: UnorderedView,
    depth_map: DepthStencil,
}

impl GBuffer {
    pub fn new(device: *mut ID3D11Device, viewport: Viewport) -> Self {
        let normal_map = Texture2D::new(
            device,
            viewport,
            1,
            DXGI_FORMAT_R32G32B32A32_FLOAT,
            D3D11_BIND_UNORDERED_ACCESS,
            0,
        );

        let world_pos_map_read = Texture2D::new(
            device,
            viewport,
            1,
            DXGI_FORMAT_R32G32B32A32_FLOAT,
            D3D11_BIND_UNORDERED_ACCESS,
            0,
        );
        let world_pos_map_read_uav = UnorderedView::for_texture_2d(device, &world_pos_map_read);

        let world_pos_map_write = Texture2D::new(
            device,
            viewport,
            1,
            DXGI_FORMAT_R32G32B32A32_FLOAT,
            D3D11_BIND_UNORDERED_ACCESS,
            0,
        );
        let world_pos_map_write_uav = UnorderedView::for_texture_2d(device, &world_pos_map_write);

        GBuffer {
            lit_scene_read: Texture2D::new(
                device,
                viewport,
                1,
                DXGI_FORMAT_R32G32B32A32_FLOAT,
                0,
                0,
            ),
            lit_scene_write: Texture2D::new(
                device,
                viewport,
                1,
                DXGI_FORMAT_R32G32B32A32_FLOAT,
                0,
                0,
            ),
            normal_map,
            world_pos_map_read,
            world_pos_map_read_uav,
            world_pos_map_write,
            world_pos_map_write_uav,
            depth_map: DepthStencil::new(device, viewport),
        }
    }

    pub fn clear(&self, devcon: *mut ID3D11DeviceContext, color: RgbaColor) {
        self.lit_scene_write.clear(devcon, color);
        self.normal_map.clear(devcon, (0., 0., 0., 0.).into());
        self.world_pos_map_write
            .clear(devcon, (0., 0., 0., 10000.).into());
        self.depth_map.clear(devcon);
    }

    pub fn shader_resources(&self) -> [*mut ID3D11ShaderResourceView; 3] {
        [
            self.lit_scene_read.shader_resource_ptr(),
            self.normal_map.shader_resource_ptr(),
            self.world_pos_map_read.shader_resource_ptr(),
        ]
    }

    pub fn render_targets(&self) -> [*mut ID3D11RenderTargetView; 3] {
        [
            self.lit_scene_write.target_view_ptr(),
            self.normal_map.target_view_ptr(),
            self.world_pos_map_write.target_view_ptr(),
        ]
    }

    pub fn read_output(&self) -> &Texture2D {
        &self.lit_scene_read
    }

    pub fn normal_map(&self) -> &Texture2D {
        &self.normal_map
    }

    pub fn world_pos_map_read(&self) -> &Texture2D {
        &self.world_pos_map_read
    }

    pub fn world_pos_map_read_uav(&self) -> &UnorderedView {
        &self.world_pos_map_read_uav
    }

    pub fn world_pos_map_write(&self) -> &Texture2D {
        &self.world_pos_map_write
    }

    pub fn world_pos_map_write_uav(&self) -> &UnorderedView {
        &self.world_pos_map_write_uav
    }

    pub fn depth_map(&self) -> &DepthStencil {
        &self.depth_map
    }

    pub fn write_output(&self) -> &Texture2D {
        &self.lit_scene_write
    }

    pub fn swap_lit(&mut self) {
        mem::swap(&mut self.lit_scene_read, &mut self.lit_scene_write);
    }

    pub fn swap_world_pos(&mut self) {
        mem::swap(&mut self.world_pos_map_read, &mut self.world_pos_map_write);
        mem::swap(
            &mut self.world_pos_map_read_uav,
            &mut self.world_pos_map_write_uav,
        );
    }
}
