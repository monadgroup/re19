use crate::animation::clip::ClipPropertyValue;
use crate::animation::property::prop;
use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::gbuffer::GBuffer;
use crate::math::{Matrix4, Quaternion, RgbColor, RgbaColor, Vector2, Vector3, Vector4};
use crate::mesh::{primitives, Mesh, VertexInserter};
use crate::object::MeshObject;
use crate::raster_state::RasterState;
use crate::renderer::clouds_renderer::{CloudsData, CloudsRenderer};
use crate::renderer::common::{BlitRenderer, GaussBlurRenderer, PostRenderer, StandardRenderer};
use crate::renderer::fluid_sim_renderer::FluidSimRenderer;
use crate::renderer::godray_renderer::GodrayRenderer;
use crate::renderer::shadow_map_renderer::ShadowMapRenderer;
use crate::renderer::RendererCollection;
use crate::resources::shader_manager::ComputeKey;
use crate::shader_view::ShaderView;
use crate::target_view::TargetView;
use crate::texture::{
    AddressMode, DepthStencil, RenderTarget2D, Sampler, ShaderResource2D, Texture2D, Texture3D,
};
use crate::unordered_view::UnorderedView;
use crate::viewport::Viewport;
use core::{f32, mem, ptr};
use winapi::shared::dxgiformat::{DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32_FLOAT};
use winapi::um::d3d11::{
    ID3D11DepthStencilView, ID3D11RenderTargetView, D3D11_BIND_CONSTANT_BUFFER,
    D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS, D3D11_BLEND_INV_SRC_ALPHA,
    D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_VIEWPORT,
};

const DIRECTIONAL_SHADOW_MAP_SIZE: (u32, u32, u32) = (128, 128, 128);

const MERIDIANS: u32 = 16;
const PYLON_BAR_RADIUS: f32 = 0.1;
const PYLON_BIG_RADIUS: f32 = 0.2;
const PYLON_SEGMENT_HEIGHT: f32 = 13.12;
const PYLON_NUM_SEGMENTS: usize = 10;
const PYLON_STRAIGHT_SEGMENTS: usize = 4;
const PYLON_BOTTOM_RADIUS: f32 = 17.45;
const PYLON_TOP_RADIUS: f32 = 3.3;
const TOP_BAR_HEIGHT: f32 = 25.567;
const TOP_BAR_RADIUS: f32 = 0.867;

// Pylon mesh generation
fn cylinder_line(
    insert: &mut VertexInserter,
    start: Vector3,
    end: Vector3,
    radius: f32,
    meridians: u32,
) {
    let transform = Matrix4::translate(start)
        * Vector3::unit_y()
            .get_rotation_to((end - start).unit(), Vector3::unit_x())
            .as_matrix()
        * Matrix4::scale(Vector3 {
            x: radius,
            y: (end - start).length() / 2.,
            z: radius,
        })
        * Matrix4::translate(Vector3 {
            x: 0.,
            y: 1.,
            z: 0.,
        });

    insert.with_transform(transform, |insert| primitives::cylinder(insert, meridians));
}

fn gen_segment_side(insert: &mut VertexInserter, bottom_radius: f32, top_radius: f32, height: f32) {
    let bottom_side_length = bottom_radius; // * f32::consts::SQRT_2;//(bottom_radius*bottom_radius * 2.).sqrt();
    let top_side_length = top_radius; // * f32::consts::SQRT_2;//(top_radius*top_radius * 2.).sqrt();
    let mid_length = (bottom_side_length / 2. + top_side_length / 2.) / 2.;

    let bottom_left = Vector3 {
        x: bottom_side_length / 2.,
        y: 0.,
        z: bottom_side_length / 2.,
    };
    let bottom_right = Vector3 {
        x: -bottom_side_length / 2.,
        y: 0.,
        z: bottom_side_length / 2.,
    };
    let bottom_midpoint = Vector3 {
        x: 0.,
        y: 0.,
        z: bottom_side_length / 2.,
    };
    let left_midpoint = Vector3 {
        x: -mid_length,
        y: height / 2.,
        z: mid_length,
    };
    let top_midpoint = Vector3 {
        x: 0.,
        y: height,
        z: top_side_length / 2.,
    };
    let right_midpoint = Vector3 {
        x: mid_length,
        y: height / 2.,
        z: mid_length,
    };

    cylinder_line(
        insert,
        bottom_left,
        bottom_right,
        PYLON_BIG_RADIUS,
        MERIDIANS,
    );
    cylinder_line(
        insert,
        bottom_midpoint,
        left_midpoint,
        PYLON_BAR_RADIUS,
        MERIDIANS,
    );
    cylinder_line(
        insert,
        bottom_midpoint,
        right_midpoint,
        PYLON_BAR_RADIUS,
        MERIDIANS,
    );
    cylinder_line(
        insert,
        left_midpoint,
        top_midpoint,
        PYLON_BAR_RADIUS,
        MERIDIANS,
    );
    cylinder_line(
        insert,
        right_midpoint,
        top_midpoint,
        PYLON_BAR_RADIUS,
        MERIDIANS,
    );
}

fn gen_segment(insert: &mut VertexInserter, bottom_radius: f32, top_radius: f32, height: f32) {
    let mut segment_side = Mesh::new(4);
    gen_segment_side(
        &mut segment_side.insert(),
        bottom_radius,
        top_radius,
        height,
    );

    insert.mesh(&segment_side);
    insert.with_transform(
        Matrix4::rotate_axis(Vector3::unit_y(), f32::consts::FRAC_PI_2),
        |insert| insert.mesh(&segment_side),
    );
    insert.with_transform(
        Matrix4::rotate_axis(Vector3::unit_y(), f32::consts::PI),
        |insert| insert.mesh(&segment_side),
    );
    insert.with_transform(
        Matrix4::rotate_axis(Vector3::unit_y(), -f32::consts::FRAC_PI_2),
        |insert| insert.mesh(&segment_side),
    );
}

fn gen_pylon_group(
    insert: &mut VertexInserter,
    bottom_radius: f32,
    top_radius: f32,
    segment_height: f32,
    num_segments: usize,
) {
    let height = segment_height * num_segments as f32;

    for segment in 0..num_segments {
        let y = segment_height * segment as f32;
        let br = bottom_radius + (top_radius - bottom_radius) * y / height;
        let tr = bottom_radius + (top_radius - bottom_radius) * (y + segment_height) / height;

        insert.with_transform(Matrix4::translate(Vector3 { x: 0., y, z: 0. }), |insert| {
            gen_segment(insert, br, tr, segment_height)
        });
    }

    // Generate corners and faces
    let bottom_v = bottom_radius / 2.; // * f32::consts::SQRT_2 / 2.;//(bottom_radius * bottom_radius * 2.).sqrt() / 2.;
    let top_v = top_radius / 2.; // * f32::consts::SQRT_2 / 2.;//(top_radius * top_radius * 2.).sqrt() / 2.;
    let lines = [
        (
            Vector3 {
                x: bottom_v,
                y: 0.,
                z: bottom_v,
            },
            Vector3 {
                x: top_v,
                y: height,
                z: top_v,
            },
        ),
        (
            Vector3 {
                x: bottom_v,
                y: 0.,
                z: -bottom_v,
            },
            Vector3 {
                x: top_v,
                y: height,
                z: -top_v,
            },
        ),
        (
            Vector3 {
                x: -bottom_v,
                y: 0.,
                z: -bottom_v,
            },
            Vector3 {
                x: -top_v,
                y: height,
                z: -top_v,
            },
        ),
        (
            Vector3 {
                x: -bottom_v,
                y: 0.,
                z: bottom_v,
            },
            Vector3 {
                x: -top_v,
                y: height,
                z: top_v,
            },
        ),
        (
            Vector3 {
                x: bottom_v,
                y: 0.,
                z: 0.,
            },
            Vector3 {
                x: top_v,
                y: height,
                z: 0.,
            },
        ),
        (
            Vector3 {
                x: -bottom_v,
                y: 0.,
                z: 0.,
            },
            Vector3 {
                x: -top_v,
                y: height,
                z: 0.,
            },
        ),
        (
            Vector3 {
                x: 0.,
                y: 0.,
                z: bottom_v,
            },
            Vector3 {
                x: 0.,
                y: height,
                z: top_v,
            },
        ),
        (
            Vector3 {
                x: 0.,
                y: 0.,
                z: -bottom_v,
            },
            Vector3 {
                x: 0.,
                y: height,
                z: -top_v,
            },
        ),
    ];

    for &(start_point, end_point) in &lines {
        cylinder_line(insert, start_point, end_point, PYLON_BIG_RADIUS, MERIDIANS);
    }
}

fn gen_pylon(insert: &mut VertexInserter) {
    gen_pylon_group(
        insert,
        PYLON_BOTTOM_RADIUS,
        PYLON_TOP_RADIUS,
        PYLON_SEGMENT_HEIGHT,
        PYLON_NUM_SEGMENTS,
    );
    insert.with_transform(
        Matrix4::translate(Vector3 {
            x: 0.,
            y: PYLON_SEGMENT_HEIGHT * PYLON_NUM_SEGMENTS as f32,
            z: 0.,
        }),
        |insert| {
            gen_pylon_group(
                insert,
                PYLON_TOP_RADIUS,
                PYLON_TOP_RADIUS,
                PYLON_SEGMENT_HEIGHT,
                PYLON_STRAIGHT_SEGMENTS,
            )
        },
    );

    let top_bar_base = Vector3 {
        x: 0.,
        y: PYLON_SEGMENT_HEIGHT * (PYLON_NUM_SEGMENTS + PYLON_STRAIGHT_SEGMENTS) as f32,
        z: 0.,
    };
    insert.with_params(
        Matrix4::translate(top_bar_base)
            * Matrix4::scale(Vector3 {
                x: PYLON_TOP_RADIUS / 2. + PYLON_BIG_RADIUS,
                y: 0.1,
                z: PYLON_TOP_RADIUS / 2. + PYLON_BIG_RADIUS,
            }),
        Vector2 { x: 0., y: 0. },
        Vector2 { x: 0., y: 0. },
        |insert| primitives::cube(insert, 0, 0),
    );

    insert.with_uv_map(
        Vector2 { x: 1., y: 1. },
        Vector2 { x: 0., y: 0. },
        |insert| {
            cylinder_line(
                insert,
                top_bar_base,
                top_bar_base
                    + Vector3 {
                        x: 0.,
                        y: TOP_BAR_HEIGHT,
                        z: 0.,
                    },
                TOP_BAR_RADIUS,
                32,
            )
        },
    )
}

#[derive(Clone, Copy)]
#[repr(C)]
struct FluidRenderData {
    // Directional light
    scaled_directional_shadow_to_world: Matrix4,
    world_to_directional_shadow: Matrix4,

    // Point light
    point_light_world_pos: Vector3,
    point_light_max_radius: f32,
    point_shadow_map_size: [u32; 3],
    point_light_radius: f32,

    // Density
    world_to_density: Matrix4,
    fluid_box_pos: Vector3,
    shadow_map_depth: u32,
    fluid_box_size: Vector3,

    use_point_light: u32,
    light_color: RgbColor,
    march_step_length: f32,
    density_multiplier: f32,
    rocket_height: f32,
}

pub struct CloudState {
    has_rendered_clouds: bool,
    clouds_tex: Texture2D,
}

impl CloudState {
    pub fn new(context: &mut CreationContext) -> Self {
        CloudState {
            has_rendered_clouds: false,
            clouds_tex: Texture2D::new(
                context.device,
                context.viewport,
                1,
                DXGI_FORMAT_R32G32B32A32_FLOAT,
                0,
                0,
            ),
        }
    }
}

pub struct LaunchScene {
    render_directional_shadow_shader: ComputeKey,
    render_point_shadow_shader: ComputeKey,

    rocket_renderer: PostRenderer,
    pylon_mesh: MeshObject,
    pylon_renderer: StandardRenderer,
    ground_plane: MeshObject,
    ground_renderer: StandardRenderer,

    fluid_render_data: Buffer<FluidRenderData>,

    fluid_renderer: PostRenderer,
    fluid_blend: BlendState,

    shadow_smp: Sampler,
    directional_shadow_map: Texture3D,
    directional_shadow_srv: ShaderView,
    directional_shadow_uav: UnorderedView,
}

impl LaunchScene {
    pub fn new(context: &mut CreationContext) -> Self {
        let mut pylon_mesh = Mesh::new(4);
        gen_pylon(
            &mut pylon_mesh.insert().transformed(Matrix4::scale(Vector3 {
                x: 0.08,
                y: 0.08,
                z: 0.08,
            })),
        );

        let mut ground_mesh = Mesh::new(4);
        primitives::quad(
            &mut ground_mesh.insert().transformed(Matrix4::scale(Vector3 {
                x: 100.,
                y: 1.,
                z: 100.,
            })),
            0,
            0,
        );

        let directional_shadow_map = Texture3D::new(
            context.device,
            DIRECTIONAL_SHADOW_MAP_SIZE.0,
            DIRECTIONAL_SHADOW_MAP_SIZE.1,
            DIRECTIONAL_SHADOW_MAP_SIZE.2,
            1,
            DXGI_FORMAT_R32_FLOAT,
            D3D11_BIND_UNORDERED_ACCESS | D3D11_BIND_SHADER_RESOURCE,
        );
        let directional_shadow_srv =
            ShaderView::for_texture_3d(context.device, &directional_shadow_map);
        let directional_shadow_uav =
            UnorderedView::for_texture_3d(context.device, &directional_shadow_map);

        LaunchScene {
            render_directional_shadow_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/render_directional_shadow.cs"),
            render_point_shadow_shader: context
                .shader_manager
                .load_shader(context.device, "fluid/render_point_shadow.cs"),

            rocket_renderer: PostRenderer::new(context, "launch_scene.ps"),
            pylon_mesh: MeshObject::new(context, &pylon_mesh),
            pylon_renderer: StandardRenderer::new(context, "identity.vs", "pylon.ps"),
            ground_plane: MeshObject::new(context, &ground_mesh),
            ground_renderer: StandardRenderer::new(context, "identity.vs", "ground.ps"),

            fluid_render_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
            fluid_renderer: PostRenderer::new(context, "fluid/render_fluid.ps"),
            fluid_blend: BlendState::new_dependent(
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

            shadow_smp: Sampler::new(
                context.device,
                D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressMode::Border(Vector4::default()),
            ),
            directional_shadow_map,
            directional_shadow_srv,
            directional_shadow_uav,
        }
    }

    fn render_shadowed_geometry(
        &mut self,
        context: &mut FrameContext,
        targets: &[*mut ID3D11RenderTargetView],
        depth_target: &DepthStencil,
        viewport: Viewport,
        render_sdf: bool,
    ) {
        let models_perf = context.perf.start_gpu_str("render models");

        self.pylon_renderer
            .render_start(context, targets, depth_target.depth_stencil_view_ptr());
        unsafe {
            (*context.devcon).RSSetViewports(
                1,
                &D3D11_VIEWPORT {
                    TopLeftX: 0.,
                    TopLeftY: 0.,
                    Width: viewport.width as f32,
                    Height: viewport.height as f32,
                    MinDepth: 0.,
                    MaxDepth: 1.,
                },
            );
        }
        let translate = Matrix4::translate(Vector3 {
            x: 7.,
            y: 0.,
            z: 0.,
        });
        let y_translate = Matrix4::translate(Vector3 {
            x: 0.,
            y: -2.,
            z: 0.,
        });
        self.pylon_mesh.render(
            context,
            Matrix4::rotate_y(f32::consts::FRAC_PI_4) * translate * y_translate,
        );
        self.pylon_mesh.render(
            context,
            Matrix4::rotate_y(f32::consts::FRAC_PI_2 + f32::consts::FRAC_PI_4)
                * translate
                * y_translate,
        );
        self.pylon_mesh.render(
            context,
            Matrix4::rotate_y(-f32::consts::FRAC_PI_4) * translate,
        );
        self.pylon_mesh.render(
            context,
            Matrix4::rotate_y(-f32::consts::FRAC_PI_2 - f32::consts::FRAC_PI_4) * translate,
        );
        unsafe {
            (*context.devcon).RSSetViewports(0, ptr::null());
        }
        self.pylon_renderer.render_end(context);

        self.ground_renderer
            .render_start(context, targets, depth_target.depth_stencil_view_ptr());
        self.ground_plane.render(context, Matrix4::default());
        self.ground_renderer.render_end(context);

        context.perf.end(models_perf);

        if render_sdf {
            let sdf_perf = context.perf.start_gpu_str("render sdf");
            unsafe {
                (*context.devcon).PSSetConstantBuffers(
                    1,
                    3,
                    &[
                        context.common.camera_buffer.ptr(),
                        self.fluid_render_data.ptr(),
                        context.common.light_buffer.ptr(),
                    ][0],
                );
                (*context.devcon).OMSetRenderTargets(
                    targets.len() as u32,
                    if targets.is_empty() {
                        ptr::null()
                    } else {
                        &targets[0]
                    },
                    depth_target.depth_stencil_view_ptr(),
                );
            }
            self.rocket_renderer
                .render_start(context, &[], None, Some(viewport), false);
            context.common.screen_quad.render(context.devcon);
            self.rocket_renderer.render_end(context, false);

            unsafe {
                (*context.devcon).PSSetConstantBuffers(
                    1,
                    3,
                    &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
                );
            }
            context.perf.end(sdf_perf);
        }
    }

    pub fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        shadow_map: &mut ShadowMapRenderer,
        fluid: &mut FluidSimRenderer,
        godray: &mut GodrayRenderer,
        clouds: &mut CloudsRenderer,
        blit: &mut BlitRenderer,
        cloud_state: &mut CloudState,
        properties: &[&[ClipPropertyValue]],
    ) {
        let light_map_pos: Vector3 = prop(properties, 0, 0);
        let light_map_x_range: Vector2 = prop(properties, 0, 1);
        let light_map_y_range: Vector2 = prop(properties, 0, 2);
        let light_map_z_range: Vector2 = prop(properties, 0, 3);
        let rays_density: f32 = prop(properties, 1, 0);

        let fluid_box_pos: Vector3 = prop(properties, 2, 0);
        let fluid_box_size: Vector3 = prop(properties, 2, 1);
        let march_step_length: f32 = prop(properties, 2, 2);

        let fluid_shadow_mode: f32 = prop(properties, 3, 0);
        let fluid_shadow_pos: Vector3 = prop(properties, 3, 1);
        let fluid_shadow_color = prop::<RgbaColor>(properties, 3, 2).premult();
        let fluid_shadow_directional_size: Vector3 = prop(properties, 3, 3);
        let fluid_shadow_point_radius: f32 = prop(properties, 3, 4);
        let fluid_shadow_point_max_radius: f32 = prop(properties, 3, 5);

        let render_sdf = prop::<f32>(properties, 4, 1) != 0.;

        let render_clouds = prop::<f32>(properties, 5, 0) != 0.;
        let clouds_y: f32 = prop(properties, 5, 1);
        let clouds_height: f32 = prop(properties, 5, 2);
        let clouds_map_offset: Vector3 = prop(properties, 5, 3);
        let clouds_color = prop::<RgbaColor>(properties, 5, 4).premult();
        let clouds_scatter_color = prop::<RgbaColor>(properties, 5, 5).premult();
        let clouds_light_direction: Quaternion = prop(properties, 5, 6);
        let clouds_opacity: f32 = prop(properties, 5, 7);
        let clouds_are_live = prop::<f32>(properties, 5, 8) != 0.;

        if (!cloud_state.has_rendered_clouds || clouds_are_live) && render_clouds {
            cloud_state.has_rendered_clouds = true;

            let clouds_perf = context.perf.start_gpu_str("clouds");
            cloud_state
                .clouds_tex
                .clear(context.devcon, RgbaColor::default());
            clouds.render(
                context,
                CloudsData {
                    map_offset: clouds_map_offset,
                    cloud_y: clouds_y,
                    sky_color: clouds_color,
                    cloud_height: clouds_height,
                    scatter_color: clouds_scatter_color,
                    cloud_opacity: clouds_opacity,
                    light_direction: clouds_light_direction.as_forward(),
                },
                io.world_pos_map_write(),
                &cloud_state.clouds_tex,
                false,
            );
            context.perf.end(clouds_perf);
        }
        if cloud_state.has_rendered_clouds && !render_clouds {
            cloud_state.has_rendered_clouds = false;
        }

        let fluid_box_scale = Vector3 {
            x: 1. / fluid_box_size.x.max(0.01),
            y: 1. / fluid_box_size.y.max(0.01),
            z: 1. / fluid_box_size.z.max(0.01),
        };
        let world_to_density_transform =
            Matrix4::scale(fluid_box_scale) * Matrix4::translate(-fluid_box_pos);

        let fluid_shadow_scale = Vector3 {
            x: 1. / fluid_shadow_directional_size.x.max(0.01),
            y: 1. / fluid_shadow_directional_size.y.max(0.01),
            z: 1. / fluid_shadow_directional_size.z.max(0.01),
        };
        let world_to_directional_shadow_transform =
            Matrix4::translate(Vector3 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            }) * (context.common.light_data.world_light_rotation
                * Quaternion::axis(Vector3::unit_y(), f32::consts::PI))
            .as_matrix()
            .inverted()
                * Matrix4::translate(Vector3 {
                    x: -0.5,
                    y: -0.5,
                    z: -0.5,
                })
                * Matrix4::scale(fluid_shadow_scale)
                * Matrix4::translate(-fluid_shadow_pos);
        let directional_shadow_to_world_transform =
            world_to_directional_shadow_transform.inverted();

        self.fluid_render_data.upload(
            context.devcon,
            FluidRenderData {
                scaled_directional_shadow_to_world: directional_shadow_to_world_transform
                    * Matrix4::scale(Vector3 {
                        x: 1. / DIRECTIONAL_SHADOW_MAP_SIZE.0 as f32,
                        y: 1. / DIRECTIONAL_SHADOW_MAP_SIZE.1 as f32,
                        z: 1. / DIRECTIONAL_SHADOW_MAP_SIZE.2 as f32,
                    }),
                world_to_directional_shadow: world_to_directional_shadow_transform,

                point_light_world_pos: fluid_shadow_pos,
                point_light_max_radius: fluid_shadow_point_max_radius,
                point_shadow_map_size: [
                    DIRECTIONAL_SHADOW_MAP_SIZE.0,
                    DIRECTIONAL_SHADOW_MAP_SIZE.1,
                    DIRECTIONAL_SHADOW_MAP_SIZE.2,
                ],
                point_light_radius: fluid_shadow_point_radius,

                world_to_density: world_to_density_transform,
                fluid_box_pos,
                shadow_map_depth: DIRECTIONAL_SHADOW_MAP_SIZE.2,
                fluid_box_size,

                use_point_light: (fluid_shadow_mode > 0.5) as u32,
                light_color: fluid_shadow_color,
                march_step_length,
                density_multiplier: prop(properties, 2, 3),
                rocket_height: prop(properties, 4, 0),
            },
        );

        // Render shadow map
        let world_to_shadow_transform = if rays_density > 0. {
            let shadow_map_perf = context.perf.start_gpu_str("shadow map");
            let (shadow_target, viewport, world_to_shadow_transform, shadow_map_state) = shadow_map
                .render_start(
                    context,
                    light_map_pos,
                    light_map_x_range,
                    light_map_y_range,
                    light_map_z_range,
                );
            self.render_shadowed_geometry(
                context,
                &[],
                shadow_target,
                Viewport {
                    width: viewport.Width as u32,
                    height: viewport.Height as u32,
                },
                render_sdf,
            );
            shadow_map.render_end(context, shadow_map_state);
            context.perf.end(shadow_map_perf);

            world_to_shadow_transform
        } else {
            Matrix4::default()
        };

        if render_clouds {
            let clouds_perf = context.perf.start_gpu_str("clouds");
            unsafe {
                (*context.devcon).OMSetBlendState(
                    self.fluid_blend.ptr(),
                    &[1., 1., 1., 1.],
                    0xFFFFFF,
                );
            }
            blit.render(context, &cloud_state.clouds_tex, io.write_output(), true);
            unsafe {
                (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
            }
            context.perf.end(clouds_perf);
        }

        self.render_shadowed_geometry(
            context,
            &io.render_targets(),
            io.depth_map(),
            context.viewport,
            render_sdf,
        );

        // Generate fluid shadow map
        let fluid_shadow_perf = context.perf.start_gpu_str("fluid shadow");
        unsafe {
            (*context.devcon).CSSetConstantBuffers(2, 1, &self.fluid_render_data.ptr());
            (*context.devcon).CSSetSamplers(0, 1, &self.shadow_smp.sampler_state_ptr());
            (*context.devcon).CSSetShaderResources(0, 1, &fluid.density_map().ptr());
            (*context.devcon).CSSetUnorderedAccessViews(
                0,
                1,
                &self.directional_shadow_uav.ptr(),
                ptr::null(),
            );
            if fluid_shadow_mode > 0.5 {
                (*context.devcon).CSSetShader(
                    context.shader_manager[self.render_point_shadow_shader].get_shader(),
                    ptr::null(),
                    0,
                );
            } else {
                (*context.devcon).CSSetShader(
                    context.shader_manager[self.render_directional_shadow_shader].get_shader(),
                    ptr::null(),
                    0,
                );
            }

            (*context.devcon).Dispatch(
                DIRECTIONAL_SHADOW_MAP_SIZE.0 / 32,
                DIRECTIONAL_SHADOW_MAP_SIZE.1 / 32,
                1,
            );

            (*context.devcon).CSSetConstantBuffers(2, 1, &ptr::null_mut());
            (*context.devcon).CSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetShaderResources(0, 1, &ptr::null_mut());
            (*context.devcon).CSSetUnorderedAccessViews(0, 1, &ptr::null_mut(), ptr::null());
            (*context.devcon).CSSetShader(ptr::null_mut(), ptr::null(), 0);
        }
        context.perf.end(fluid_shadow_perf);

        // Render fluid
        let fluid_render_perf = context.perf.start_gpu_str("fluid render");
        unsafe {
            (*context.devcon).PSSetSamplers(0, 1, &self.shadow_smp.sampler_state_ptr());
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[
                    fluid.density_map().ptr(),
                    self.directional_shadow_srv.ptr(),
                    io.world_pos_map_write().shader_resource_ptr(),
                ][0],
            );
            (*context.devcon).PSSetConstantBuffers(
                1,
                2,
                &[
                    context.common.camera_buffer.ptr(),
                    self.fluid_render_data.ptr(),
                ][0],
            );
            (*context.devcon).OMSetBlendState(self.fluid_blend.ptr(), &[1., 1., 1., 1.], 0xFFFFFF);
        }
        self.fluid_renderer
            .render(context, io.write_output(), true, false);
        unsafe {
            (*context.devcon).PSSetSamplers(0, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(
                0,
                3,
                &[ptr::null_mut(), ptr::null_mut(), ptr::null_mut()][0],
            );
            (*context.devcon).PSSetConstantBuffers(1, 2, &[ptr::null_mut(), ptr::null_mut()][0]);
            (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
        }
        context.perf.end(fluid_render_perf);

        // Render godrays
        if rays_density > 0. {
            let godrays_perf = context.perf.start_gpu_str("godrays");
            godray.render(
                context,
                world_to_shadow_transform,
                rays_density,
                prop::<f32>(properties, 1, 1) as u32,
                prop(properties, 1, 2),
                prop(properties, 1, 3),
                io.world_pos_map_write(),
                shadow_map.shadow_map(),
                Some((
                    world_to_directional_shadow_transform,
                    &self.directional_shadow_srv,
                )),
                io.write_output(),
            );
            context.perf.end(godrays_perf);
        }
    }
}
