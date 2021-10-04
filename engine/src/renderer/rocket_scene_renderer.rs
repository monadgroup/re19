use crate::animation::clip::ClipPropertyValue;
use crate::animation::property::prop;
use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::gbuffer::GBuffer;
use crate::math::random::rand_float;
use crate::math::{Matrix4, Quaternion, RgbColor, RgbaColor, Vector3, Vector4};
use crate::renderer::common::PostRenderer;
use crate::resources::shader_manager::ComputeKey;
use crate::shader_view::ShaderView;
use crate::texture::{
    generators, AddressMode, RenderTarget2D, Sampler, ShaderResource2D, Texture3D,
};
use crate::unordered_view::UnorderedView;
use core::{f32, mem, ptr};
use winapi::shared::dxgiformat::{DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_FLOAT};
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS,
    D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR,
};

const DENSITY_RESOLUTION: (u32, u32, u32) = (512, 512, 512);
const DIRECTIONAL_SHADOW_RESOLUTION: (u32, u32, u32) = (128, 128, 128);
const POINT_SHADOW_RESOLUTION: (u32, u32, u32) = (256, 256, 256);
const LIGHT_MAP_RESOLUTION: (u32, u32, u32) = (128, 128, 128);
const LIGHT_MAP_MIP_COUNT: usize = 5;

#[derive(Clone, Copy)]
#[repr(C)]
struct SceneData {
    pub density_to_world: Matrix4,
    pub scaled_density_to_world: Matrix4,
    pub world_to_density: Matrix4,
    pub world_to_light_map: Matrix4,
    pub scaled_light_map_to_world: Matrix4,
    pub world_to_directional_shadow: Matrix4,
    pub scaled_directional_shadow_to_world: Matrix4,
    pub directional_shadow_map_size: [u32; 3],
    pub point_light_radius: f32,
    pub point_shadow_map_size: [u32; 3],
    pub point_light_max_radius: f32,
    pub light_map_size: [u32; 3],
    pub light_blur_size: f32,
    pub world_density_pos: Vector3,
    pub _pad0: u32,
    pub world_density_size: Vector3,
    pub _pad1: u32,
    pub rocket_base_pos: Vector3,
    pub _pad2: u32,
    pub directional_light_direction: Vector3,
    pub _pad3: u32,
    pub point_light_world_pos: Vector3,
    pub _pad4: u32,
    pub directional_light_color: RgbColor,
    pub _pad5: u32,
    pub point_light_color: RgbColor,
    pub point_light_flicker: f32,
    pub ambient_light_color: RgbColor,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct DownscalePassData {
    pub size_divisor: u32,
}

struct LightMap {
    _map: Texture3D,
    srv: ShaderView,

    mip_uavs: [UnorderedView; LIGHT_MAP_MIP_COUNT],
    mip_srvs: [ShaderView; LIGHT_MAP_MIP_COUNT],
}

impl LightMap {
    pub fn new(context: &mut CreationContext) -> Self {
        let map = Texture3D::new(
            context.device,
            LIGHT_MAP_RESOLUTION.0,
            LIGHT_MAP_RESOLUTION.1,
            LIGHT_MAP_RESOLUTION.2,
            LIGHT_MAP_MIP_COUNT as u32,
            DXGI_FORMAT_R32G32_FLOAT,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let srv = ShaderView::for_texture_3d(context.device, &map);

        let mut mip_uavs: [UnorderedView; LIGHT_MAP_MIP_COUNT] = unsafe { mem::uninitialized() };
        let mut mip_srvs: [ShaderView; LIGHT_MAP_MIP_COUNT] = unsafe { mem::uninitialized() };
        for mip_index in 0..LIGHT_MAP_MIP_COUNT {
            let mip_uav = UnorderedView::for_texture_3d_layer(
                context.device,
                &map,
                DXGI_FORMAT_R32G32_FLOAT,
                mip_index as u32,
            );
            let mip_srv = ShaderView::for_texture_3d_layer(
                context.device,
                &map,
                DXGI_FORMAT_R32G32_FLOAT,
                mip_index as u32,
            );

            unsafe {
                ptr::write(&mut mip_uavs[mip_index], mip_uav);
                ptr::write(&mut mip_srvs[mip_index], mip_srv);
            }
        }

        LightMap {
            _map: map,
            srv,
            mip_uavs,
            mip_srvs,
        }
    }
}

pub struct RocketScene {
    has_prepared_data: bool,
    last_flicker_rand: f32,

    objects_renderer: PostRenderer,

    build_static_density_shader: ComputeKey,
    build_directional_shadow_shader: ComputeKey,
    build_point_shadow_shader: ComputeKey,
    //downscale_directional_shadow_shader: ComputeKey,
    //downscale_point_shadow_shader: ComputeKey,
    build_light_map_shader: ComputeKey,
    downscale_light_map_shader: ComputeKey,
    coallesce_light_map_shader: ComputeKey,
    density_renderer: PostRenderer,

    scene_data: Buffer<SceneData>,
    downscale_pass_data: Buffer<DownscalePassData>,

    volume_blend_state: BlendState,

    point_light_sampler: Sampler,
    volume_sampler: Sampler,
    noise_sampler: Sampler,

    _noise_volume: Texture3D,
    noise_volume_srv: ShaderView,

    _density_map: Texture3D,
    density_map_srv: ShaderView,
    density_map_uav: UnorderedView,

    _directional_shadow_map: Texture3D,
    directional_shadow_map_srv: ShaderView,
    directional_shadow_map_uav: UnorderedView,

    _point_shadow_map: Texture3D,
    point_shadow_map_srv: ShaderView,
    point_shadow_map_uav: UnorderedView,

    light_map: LightMap,

    _coallesced_light_map: Texture3D,
    coallesced_light_map_srv: ShaderView,
    coallesced_light_map_uav: UnorderedView,
    //directional_shadow_map: ShadowMap,
    //point_shadow_map: ShadowMap,

    //light_map: Texture3D,
    //light_map_srv: ShaderView,
    //light_map_uav: UnorderedView,
}

impl RocketScene {
    pub fn new(context: &mut CreationContext) -> Self {
        let noise_volume = generators::noise_volume(context.device, 256, 256, 256);
        let noise_volume_srv = ShaderView::for_texture_3d(context.device, &noise_volume);

        let density_map = Texture3D::new(
            context.device,
            DENSITY_RESOLUTION.0,
            DENSITY_RESOLUTION.1,
            DENSITY_RESOLUTION.2,
            1,
            DXGI_FORMAT_R32_FLOAT,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let density_map_srv = ShaderView::for_texture_3d(context.device, &density_map);
        let density_map_uav = UnorderedView::for_texture_3d(context.device, &density_map);

        let directional_shadow_map = Texture3D::new(
            context.device,
            DIRECTIONAL_SHADOW_RESOLUTION.0,
            DIRECTIONAL_SHADOW_RESOLUTION.1,
            DIRECTIONAL_SHADOW_RESOLUTION.2,
            1,
            DXGI_FORMAT_R32_FLOAT,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let directional_shadow_map_srv =
            ShaderView::for_texture_3d(context.device, &directional_shadow_map);
        let directional_shadow_map_uav =
            UnorderedView::for_texture_3d(context.device, &directional_shadow_map);

        let point_shadow_map = Texture3D::new(
            context.device,
            POINT_SHADOW_RESOLUTION.0,
            POINT_SHADOW_RESOLUTION.1,
            POINT_SHADOW_RESOLUTION.2,
            1,
            DXGI_FORMAT_R32_FLOAT,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let point_shadow_map_srv = ShaderView::for_texture_3d(context.device, &point_shadow_map);
        let point_shadow_map_uav = UnorderedView::for_texture_3d(context.device, &point_shadow_map);

        let coallesced_light_map = Texture3D::new(
            context.device,
            LIGHT_MAP_RESOLUTION.0,
            LIGHT_MAP_RESOLUTION.1,
            LIGHT_MAP_RESOLUTION.2,
            1,
            DXGI_FORMAT_R32G32_FLOAT,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let coallesced_light_map_srv =
            ShaderView::for_texture_3d(context.device, &coallesced_light_map);
        let coallesced_light_map_uav =
            UnorderedView::for_texture_3d(context.device, &coallesced_light_map);

        RocketScene {
            has_prepared_data: false,
            last_flicker_rand: 0.,

            objects_renderer: PostRenderer::new(context, "rocket_cloud_scene.ps"),

            build_static_density_shader: context
                .shader_manager
                .load_shader(context.device, "volume/build_static_density.cs"),
            build_directional_shadow_shader: context
                .shader_manager
                .load_shader(context.device, "volume/build_directional_shadow.cs"),
            build_point_shadow_shader: context
                .shader_manager
                .load_shader(context.device, "volume/build_point_shadow.cs"),
            //downscale_directional_shadow_shader: context
            //    .shader_manager
            //    .load_shader(context.device, "volume/downscale_directional_shadow.cs"),
            //downscale_point_shadow_shader: context
            //    .shader_manager
            //    .load_shader(context.device, "volume/downscale_point_shadow.cs"),
            build_light_map_shader: context
                .shader_manager
                .load_shader(context.device, "volume/build_light_map.cs"),
            downscale_light_map_shader: context
                .shader_manager
                .load_shader(context.device, "volume/downscale_light_map.cs"),
            coallesce_light_map_shader: context
                .shader_manager
                .load_shader(context.device, "volume/coallesce_light_map.cs"),
            density_renderer: PostRenderer::new(context, "volume/render_density.ps"),

            scene_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            downscale_pass_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),

            volume_blend_state: BlendState::new_dependent(
                context.device,
                false,
                BlendRenderTargetConfig::enabled(
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                ),
            ),

            point_light_sampler: Sampler::new(
                context.device,
                D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressMode::Wrap,
            ),
            volume_sampler: Sampler::new(
                context.device,
                D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressMode::Border(Vector4::default()),
            ),
            noise_sampler: Sampler::new(
                context.device,
                D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressMode::Wrap,
            ),

            _noise_volume: noise_volume,
            noise_volume_srv,

            _density_map: density_map,
            density_map_srv,
            density_map_uav,

            _directional_shadow_map: directional_shadow_map,
            directional_shadow_map_srv,
            directional_shadow_map_uav,

            _point_shadow_map: point_shadow_map,
            point_shadow_map_srv,
            point_shadow_map_uav,

            //directional_shadow_map: LightMap::new(context),
            //point_shadow_map: LightMap::new(context),

            //light_map,
            //light_map_srv,
            //light_map_uav,
            light_map: LightMap::new(context),

            _coallesced_light_map: coallesced_light_map,
            coallesced_light_map_srv,
            coallesced_light_map_uav,
        }
    }

    fn render_density_map(&mut self, context: &mut FrameContext) {
        if self.has_prepared_data {
            return;
        }

        unsafe {
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.density_map_uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetSamplers(0, 1, &self.noise_sampler.sampler_state_ptr());
            (*context.devcon).CSSetShaderResources(0, 1, &self.noise_volume_srv.ptr());
            (*context.devcon).CSSetShader(
                context.shader_manager[self.build_static_density_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(
                DENSITY_RESOLUTION.0 / 32,
                DENSITY_RESOLUTION.1 / 32,
                DENSITY_RESOLUTION.2,
            );

            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn render_shadow_maps(&mut self, context: &mut FrameContext) {
        if !self.has_prepared_data {
            // Render the shadow map for the directional light
            let dir_shadow_perf = context.perf.start_gpu_str("directional shadow");
            unsafe {
                (*context.devcon).CSSetSamplers(0, 1, &self.volume_sampler.sampler_state_ptr());
                (*context.devcon).CSSetShaderResources(0, 1, &self.density_map_srv.ptr());
                (*context.devcon).CSSetUnorderedAccessViews(
                    0,
                    1,
                    &self.directional_shadow_map_uav.ptr(),
                    ptr::null(),
                );
                (*context.devcon).CSSetShader(
                    context.shader_manager[self.build_directional_shadow_shader].get_shader(),
                    ptr::null(),
                    0,
                );

                (*context.devcon).Dispatch(
                    DIRECTIONAL_SHADOW_RESOLUTION.0 / 32,
                    DIRECTIONAL_SHADOW_RESOLUTION.1 / 32,
                    1,
                );

                (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
                (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
                (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
                (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
            }
            context.perf.end(dir_shadow_perf);
        }

        // Render the shadow map for the point light
        let point_shadow_perf = context.perf.start_gpu_str("point shadow");
        unsafe {
            (*context.devcon).CSSetSamplers(0, 1, &self.volume_sampler.sampler_state_ptr());
            (*context.devcon).CSSetShaderResources(0, 1, &self.density_map_srv.ptr());
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.point_shadow_map_uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.build_point_shadow_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(
                POINT_SHADOW_RESOLUTION.0 / 32,
                POINT_SHADOW_RESOLUTION.1 / 32,
                1,
            );

            (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
        context.perf.end(point_shadow_perf);
    }

    fn render_light_map(&mut self, context: &mut FrameContext) {
        // Build the base lightmap level
        let build_light_map_perf = context.perf.start_gpu_str("build light map");
        unsafe {
            (*context.devcon).CSSetSamplers(
                0,
                2,
                &[
                    self.volume_sampler.sampler_state_ptr(),
                    self.point_light_sampler.sampler_state_ptr(),
                ][0],
            );
            (*context.devcon).CSSetShaderResources(
                0,
                2,
                &[
                    self.directional_shadow_map_srv.ptr(),
                    self.point_shadow_map_srv.ptr(),
                ][0],
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.light_map.mip_uavs[0].ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.build_light_map_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(
                LIGHT_MAP_RESOLUTION.0 / 32,
                LIGHT_MAP_RESOLUTION.1 / 32,
                LIGHT_MAP_RESOLUTION.2,
            );

            (*context.devcon).CSSetSamplers(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).CSSetShaderResources(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
        context.perf.end(build_light_map_perf);

        // Downscale/blur each mipmap
        let blur_light_map_perf = context.perf.start_gpu_str("blur light map");
        unsafe {
            (*context.devcon).CSSetSamplers(0, 1, &self.volume_sampler.sampler_state_ptr());
            (*context.devcon).CSSetShader(
                context.shader_manager[self.downscale_light_map_shader].get_shader(),
                ptr::null(),
                0,
            );
        }
        let mut divisor_factor = 2;
        for target_mip in 1..LIGHT_MAP_MIP_COUNT {
            self.downscale_pass_data.upload(
                context.devcon,
                DownscalePassData {
                    size_divisor: divisor_factor,
                },
            );

            unsafe {
                (*context.devcon).CSSetConstantBuffers(0, 1, &self.downscale_pass_data.ptr());
                (*context.devcon).CSSetShaderResources(
                    0,
                    1,
                    &self.light_map.mip_srvs[target_mip - 1].ptr(),
                );
                (*context.devcon).CSSetUnorderedAccessViews(
                    0,
                    1,
                    &self.light_map.mip_uavs[target_mip].ptr(),
                    ptr::null(),
                );

                (*context.devcon).Dispatch(
                    (LIGHT_MAP_RESOLUTION.0 / divisor_factor) / 4,
                    (LIGHT_MAP_RESOLUTION.1 / divisor_factor) / 4,
                    LIGHT_MAP_RESOLUTION.2 / divisor_factor,
                );

                (*context.devcon).CSSetConstantBuffers(0, 1, &ptr::null_mut());
                (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
                (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            }

            divisor_factor <<= 1;
        }
        unsafe {
            (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
        context.perf.end(blur_light_map_perf);

        // Coallesce the light map back into one texture
        let coallesce_perf = context.perf.start_gpu_str("coallesce");
        unsafe {
            (*context.devcon).CSSetSamplers(0, 1, &self.volume_sampler.sampler_state_ptr());
            (*context.devcon).CSSetShaderResources(0, 1, &self.light_map.srv.ptr());
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.coallesced_light_map_uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.coallesce_light_map_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(
                LIGHT_MAP_RESOLUTION.0 / 32,
                LIGHT_MAP_RESOLUTION.1 / 32,
                LIGHT_MAP_RESOLUTION.2,
            );

            (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
        context.perf.end(coallesce_perf);
    }

    fn render_rocket(&mut self, io: &mut GBuffer, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &context.common.camera_buffer.ptr());
            (*context.devcon).PSSetSamplers(
                0,
                2,
                &[
                    self.volume_sampler.sampler_state_ptr(),
                    self.noise_sampler.sampler_state_ptr(),
                ][0],
            );
            (*context.devcon).PSSetShaderResources(
                0,
                2,
                &[
                    self.coallesced_light_map_srv.ptr(),
                    self.noise_volume_srv.ptr(),
                ][0],
            );
        }
        self.objects_renderer.render_start(
            context,
            &[
                io.write_output().target_view_ptr(),
                io.normal_map().target_view_ptr(),
                io.world_pos_map_write().target_view_ptr(),
            ],
            Some(io.depth_map()),
            Some(io.write_output().size()),
            false,
        );
        context.common.screen_quad.render(context.devcon);
        self.objects_renderer.render_end(context, false);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetSamplers(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).PSSetShaderResources(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
        }
    }

    fn render_volume_scene(&mut self, io: &mut GBuffer, context: &mut FrameContext) {
        // Render the scene
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &context.common.camera_buffer.ptr());
            (*context.devcon).PSSetSamplers(0, 1, &self.volume_sampler.sampler_state_ptr());
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[
                    self.density_map_srv.ptr(),
                    self.coallesced_light_map_srv.ptr(),
                    io.world_pos_map_write().shader_resource_ptr(),
                ][0],
            );
            (*context.devcon).OMSetBlendState(
                self.volume_blend_state.ptr(),
                &[1., 1., 1., 1.],
                0xFFFFFF,
            );
        }
        self.density_renderer
            .render(context, io.write_output(), true, false);
        unsafe {
            (*context.devcon).PSSetConstantBuffers(1, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
        }
    }

    pub fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        properties: &[&[ClipPropertyValue]],
    ) {
        // Read properties
        let density_vol_pos: Vector3 = prop(properties, 0, 0);
        let density_vol_size: Vector3 = prop(properties, 0, 1);
        let light_map_blur: f32 = prop(properties, 0, 2);

        let directional_light_dir: Quaternion = prop(properties, 1, 0);
        let directional_light_color = prop::<RgbaColor>(properties, 1, 1).premult();
        let directional_light_vol_pos: Vector3 = prop(properties, 1, 2);
        let directional_light_vol_size: Vector3 = prop(properties, 1, 3);

        let point_light_pos: Vector3 = prop(properties, 2, 0);
        let point_light_color = prop::<RgbaColor>(properties, 2, 1).premult();
        let point_light_radius: f32 = prop(properties, 2, 2);
        let point_light_max_radius: f32 = prop(properties, 2, 3);

        // adjust the point light color based on randomness
        self.last_flicker_rand = (rand_float(0., 1.) + self.last_flicker_rand) / 2.;
        let point_light_flicker = 1.; //0.8 + self.last_flicker_rand * 0.2;
        let point_light_color = point_light_color.with_a(point_light_flicker).premult();

        let ambient_light_color = prop::<RgbaColor>(properties, 3, 0).premult();

        let rocket_base_pos: Vector3 = prop(properties, 4, 0);
        let rocket_enabled = prop::<f32>(properties, 4, 1) != 0.;

        let density_vol_scale = Vector3 {
            x: 1. / density_vol_size.x.max(0.01),
            y: 1. / density_vol_size.y.max(0.01),
            z: 1. / density_vol_size.z.max(0.01),
        };
        let world_to_density_transform =
            Matrix4::scale(density_vol_scale) * Matrix4::translate(-density_vol_pos);
        let density_to_world_transform = world_to_density_transform.inverted();

        let directional_light_vol_scale = Vector3 {
            x: 1. / directional_light_vol_size.x.max(0.01),
            y: 1. / directional_light_vol_size.y.max(0.01),
            z: 1. / directional_light_vol_size.z.max(0.01),
        };

        // Calculate the matrix to transform from the world to the shadow volume and back
        let world_to_directional_shadow_transform = Matrix4::translate(Vector3 {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        }) * (directional_light_dir
            * Quaternion::axis(Vector3::unit_y(), -f32::consts::FRAC_PI_2))
        .as_matrix()
        .inverted()
            * Matrix4::translate(Vector3 {
                x: -0.5,
                y: -0.5,
                z: -0.5,
            })
            * Matrix4::scale(directional_light_vol_scale)
            * Matrix4::translate(-directional_light_vol_pos);
        let directional_shadow_to_world_transform =
            world_to_directional_shadow_transform.inverted();

        // Upload the scene data buffer
        self.scene_data.upload(
            context.devcon,
            SceneData {
                density_to_world: density_to_world_transform,
                scaled_density_to_world: density_to_world_transform
                    * Matrix4::scale(Vector3 {
                        x: 1. / DENSITY_RESOLUTION.0 as f32,
                        y: 1. / DENSITY_RESOLUTION.1 as f32,
                        z: 1. / DENSITY_RESOLUTION.2 as f32,
                    }),
                world_to_density: world_to_density_transform,
                world_to_light_map: world_to_density_transform,
                scaled_light_map_to_world: density_to_world_transform
                    * Matrix4::scale(Vector3 {
                        x: 1. / LIGHT_MAP_RESOLUTION.0 as f32,
                        y: 1. / LIGHT_MAP_RESOLUTION.1 as f32,
                        z: 1. / LIGHT_MAP_RESOLUTION.2 as f32,
                    }),
                world_to_directional_shadow: world_to_directional_shadow_transform,
                scaled_directional_shadow_to_world: directional_shadow_to_world_transform
                    * Matrix4::scale(Vector3 {
                        x: 1. / DIRECTIONAL_SHADOW_RESOLUTION.0 as f32,
                        y: 1. / DIRECTIONAL_SHADOW_RESOLUTION.1 as f32,
                        z: 1. / DIRECTIONAL_SHADOW_RESOLUTION.2 as f32,
                    }),
                directional_shadow_map_size: [
                    DIRECTIONAL_SHADOW_RESOLUTION.0,
                    DIRECTIONAL_SHADOW_RESOLUTION.1,
                    DIRECTIONAL_SHADOW_RESOLUTION.2,
                ],
                point_light_radius,
                point_shadow_map_size: [
                    POINT_SHADOW_RESOLUTION.0,
                    POINT_SHADOW_RESOLUTION.1,
                    POINT_SHADOW_RESOLUTION.2,
                ],
                point_light_max_radius,
                light_map_size: [
                    LIGHT_MAP_RESOLUTION.0,
                    LIGHT_MAP_RESOLUTION.1,
                    LIGHT_MAP_RESOLUTION.2,
                ],
                light_blur_size: light_map_blur,
                world_density_pos: density_vol_pos,
                _pad0: 0,
                world_density_size: density_vol_size,
                _pad1: 0,
                rocket_base_pos,
                _pad2: 0,
                directional_light_direction: Vector3 {
                    x: 1.,
                    y: 0.,
                    z: 0.,
                } * directional_light_dir,
                _pad3: 0,
                point_light_world_pos: rocket_base_pos + point_light_pos, //point_light_pos,
                _pad4: 0,
                directional_light_color,
                _pad5: 0,
                point_light_color,
                point_light_flicker,
                ambient_light_color,
            },
        );
        unsafe {
            (*context.devcon).PSSetConstantBuffers(2, 1, &self.scene_data.ptr());
            (*context.devcon).CSSetConstantBuffers(2, 1, &self.scene_data.ptr());
        }

        perf!(context, "build density", self.render_density_map(context));
        self.render_shadow_maps(context);
        self.render_light_map(context);
        if rocket_enabled {
            perf!(context, "render objects", self.render_rocket(io, context));
        }
        perf!(
            context,
            "render volume",
            self.render_volume_scene(io, context)
        );

        unsafe {
            (*context.devcon).PSSetConstantBuffers(2, 1, &ptr::null_mut());
            (*context.devcon).CSSetConstantBuffers(2, 1, &ptr::null_mut());
        }

        self.has_prepared_data = true;
    }
}
