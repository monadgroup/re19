use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::{RgbaColor, Vector3, Vector4};
use crate::resources::shader_manager::ComputeKey;
use crate::shader_view::ShaderView;
use crate::texture::{AddressMode, Sampler, Texture3D};
use crate::unordered_view::UnorderedView;
use core::{mem, ptr};
use winapi::shared::dxgiformat::{
    DXGI_FORMAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_FLOAT,
    DXGI_FORMAT_R32_SINT,
};
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS,
    D3D11_FILTER_MIN_MAG_MIP_LINEAR,
};

const MAP_SIZE: (u32, u32, u32) = (128, 128, 128);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FluidProperties {
    pub input_pos: Vector3,
    pub vorticity_strength: f32,
    pub input_radius: Vector3,
    pub density_amount: f32,
    pub velocity_amount: Vector3,
    pub density_dissipation: f32,
    pub density_buoyancy: f32,
    pub density_weight: f32,
    pub temperature_amount: f32,
    pub temperature_dissipation: f32,
    pub velocity_dissipation: f32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct FluidData {
    map_size: [i32; 3],
    delta_time: f32,
    props: FluidProperties,
}

struct Map {
    tex: Texture3D,
    srv: ShaderView,
    uav: UnorderedView,
}

impl Map {
    fn new(context: &mut CreationContext, format: DXGI_FORMAT) -> Self {
        let tex = Texture3D::new(
            context.device,
            MAP_SIZE.0,
            MAP_SIZE.1,
            MAP_SIZE.2,
            1,
            format,
            D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
        );
        let srv = ShaderView::for_texture_3d(context.device, &tex);
        let uav = UnorderedView::for_texture_3d(context.device, &tex);

        Map { tex, srv, uav }
    }
}

pub struct FluidSimRenderer {
    compute_boundary_shader: ComputeKey,
    apply_advection_shader: ComputeKey,
    apply_buoyancy_shader: ComputeKey,
    apply_impulse_shader: ComputeKey,
    compute_vorticity_shader: ComputeKey,
    compute_confinement_shader: ComputeKey,
    compute_divergence_shader: ComputeKey,
    compute_pressure_shader: ComputeKey,
    compute_projection_shader: ComputeKey,

    fluid_data: Buffer<FluidData>,

    boundary_map: Map,
    velocity_map_read: Map,
    velocity_map_write: Map,
    density_temperature_map_read: Map,
    density_temperature_map_write: Map,
    pressure_map_read: Map,
    pressure_map_write: Map,
    vorticity_map: Map,
    divergence_map: Map,
}

impl FluidSimRenderer {
    pub fn new(context: &mut CreationContext) -> Self {
        FluidSimRenderer {
            compute_boundary_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_boundary.cs"),
            apply_advection_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/apply_advection.cs"),
            apply_buoyancy_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/apply_buoyancy.cs"),
            apply_impulse_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/apply_impulse.cs"),
            compute_vorticity_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_vorticity.cs"),
            compute_confinement_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_confinement.cs"),
            compute_divergence_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_divergence.cs"),
            compute_pressure_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_pressure.cs"),
            compute_projection_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/compute_projection.cs"),

            fluid_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),

            boundary_map: Map::new(context, DXGI_FORMAT_R32_SINT),
            velocity_map_read: Map::new(context, DXGI_FORMAT_R32G32B32A32_FLOAT),
            velocity_map_write: Map::new(context, DXGI_FORMAT_R32G32B32A32_FLOAT),
            density_temperature_map_read: Map::new(context, DXGI_FORMAT_R32G32_FLOAT),
            density_temperature_map_write: Map::new(context, DXGI_FORMAT_R32G32_FLOAT),
            pressure_map_read: Map::new(context, DXGI_FORMAT_R32_FLOAT),
            pressure_map_write: Map::new(context, DXGI_FORMAT_R32_FLOAT),
            vorticity_map: Map::new(context, DXGI_FORMAT_R32G32B32A32_FLOAT),
            divergence_map: Map::new(context, DXGI_FORMAT_R32_FLOAT),
        }
    }

    fn compute_boundary(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.boundary_map.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_boundary_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn apply_advection(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(
                0,
                3,
                &[
                    self.boundary_map.srv.ptr(),
                    self.velocity_map_read.srv.ptr(),
                    self.density_temperature_map_read.srv.ptr(),
                ][0],
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                2,
                &[
                    self.velocity_map_write.uav.ptr(),
                    self.density_temperature_map_write.uav.ptr(),
                ][0],
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.apply_advection_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(
                0,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                2,
                &[ptr::null_mut(), ptr::null_mut()][0],
                ptr::null(),
            );
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }

        mem::swap(&mut self.velocity_map_read, &mut self.velocity_map_write);
        mem::swap(
            &mut self.density_temperature_map_read,
            &mut self.density_temperature_map_write,
        );
    }

    fn apply_buoyancy(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(
                0,
                1,
                &self.density_temperature_map_read.srv.ptr(),
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.velocity_map_read.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.apply_buoyancy_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn apply_impulse(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                2,
                &[
                    self.density_temperature_map_read.uav.ptr(),
                    self.velocity_map_read.uav.ptr(),
                ][0],
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.apply_impulse_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                2,
                &[ptr::null_mut(), ptr::null_mut()][0],
                ptr::null(),
            );
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn compute_vorticity(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(0, 1, &self.velocity_map_read.srv.ptr());
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.vorticity_map.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_vorticity_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn compute_confinement(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(0, 1, &self.vorticity_map.srv.ptr());
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.velocity_map_read.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_confinement_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn compute_divergence(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(
                0,
                2,
                &[
                    self.boundary_map.srv.ptr(),
                    self.velocity_map_read.srv.ptr(),
                ][0],
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.divergence_map.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_divergence_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn compute_pressure(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_pressure_shader].get_shader(),
                ptr::null(),
                0,
            );
        }
        for _ in 0..10 {
            unsafe {
                (*context.devcon).CSSetShaderResources(
                    0,
                    3,
                    &[
                        self.boundary_map.srv.ptr(),
                        self.divergence_map.srv.ptr(),
                        self.pressure_map_read.srv.ptr(),
                    ][0],
                );
                (*context.devcon).CSSetUnorderedAccessViews(
                    0,
                    1,
                    &self.pressure_map_write.uav.ptr(),
                    ptr::null(),
                );

                (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

                (*context.devcon).CSSetShaderResources(
                    0,
                    3,
                    &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
                );
                (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            }

            mem::swap(&mut self.pressure_map_read, &mut self.pressure_map_write);
        }
        unsafe {
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    fn compute_projection(&mut self, context: &mut FrameContext) {
        unsafe {
            (*context.devcon).CSSetShaderResources(
                0,
                2,
                &[
                    self.boundary_map.srv.ptr(),
                    self.pressure_map_read.srv.ptr(),
                ][0],
            );
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.velocity_map_read.uav.ptr(),
                ptr::null(),
            );
            (*context.devcon).CSSetShader(
                context.shader_manager[self.compute_projection_shader].get_shader(),
                ptr::null(),
                0,
            );

            (*context.devcon).Dispatch(MAP_SIZE.0 / 32, MAP_SIZE.1 / 32, MAP_SIZE.2);

            (*context.devcon).CSSetShaderResources(0, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
    }

    pub fn clear(&mut self, context: &mut FrameContext) {
        self.velocity_map_read
            .uav
            .clear(context.devcon, RgbaColor::default());
        self.density_temperature_map_read
            .uav
            .clear(context.devcon, RgbaColor::default());
        self.pressure_map_read
            .uav
            .clear(context.devcon, RgbaColor::default());
    }

    pub fn run(&mut self, context: &mut FrameContext, props: FluidProperties) {
        self.fluid_data.upload(
            context.devcon,
            FluidData {
                map_size: [MAP_SIZE.0 as i32, MAP_SIZE.1 as i32, MAP_SIZE.2 as i32],
                delta_time: 0.1, //context.delta_seconds,
                props,
            },
        );

        unsafe {
            (*context.devcon).CSSetConstantBuffers(0, 1, &self.fluid_data.ptr());
        }
        self.compute_boundary(context);
        self.apply_advection(context);
        self.apply_buoyancy(context);
        self.apply_impulse(context);
        self.compute_vorticity(context);
        self.compute_confinement(context);
        self.compute_divergence(context);
        self.compute_pressure(context);
        self.compute_projection(context);
        unsafe {
            (*context.devcon).CSSetConstantBuffers(0, 1, &ptr::null_mut());
        }
    }

    pub fn density_map(&self) -> &ShaderView {
        &self.density_temperature_map_read.srv
    }
}
