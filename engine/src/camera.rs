use crate::frame_context::CommonData;
use crate::math::{Matrix4, Quaternion, Ray, Vector3, Vector4};
use crate::viewport::Viewport;
use core::f32::consts;
use winapi::um::d3d11::ID3D11DeviceContext;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CameraBuffer {
    pub cam_position: Vector3,
    pub cam_fov_radians: f32,
    pub cam_direction: Vector4,
    pub z_range: Vector4,
    pub view_matrix: Matrix4,
    pub proj_matrix: Matrix4,
    pub view_proj_matrix: Matrix4,
    pub last_matrix: Matrix4,
    pub inv_view_matrix: Matrix4,
    pub inv_proj_matrix: Matrix4,
    pub norm_view_matrix: Matrix4,
}

pub trait Camera {
    fn as_buffer(&self) -> CameraBuffer;

    fn upload(&self, devcon: *mut ID3D11DeviceContext, common: &mut CommonData) {
        let buffer = self.as_buffer();
        common.camera_data = buffer;
        common.camera_buffer.upload(devcon, buffer);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PerspectiveCamera {
    pub base_pos: Vector3,
    pub gymbal_dir: Quaternion,
    pub arm_length: f32,
    pub camera_dir: Quaternion,

    pub fov: f32,
    pub near_z: f32,
    pub far_z: f32,

    end_ray: Ray,
    view_matrix: Matrix4,
    proj_matrix: Matrix4,
    view_proj_matrix: Matrix4,
    last_matrix: Matrix4,
    inv_view_matrix: Matrix4,
    inv_proj_matrix: Matrix4,
    norm_view_matrix: Matrix4,
}

impl PerspectiveCamera {
    pub fn new(
        base_pos: Vector3,
        gymbal_dir: Quaternion,
        arm_length: f32,
        camera_dir: Quaternion,
        fov: f32,
        near_z: f32,
        far_z: f32,
    ) -> Self {
        PerspectiveCamera {
            base_pos,
            gymbal_dir,
            arm_length,
            camera_dir,
            fov,
            near_z,
            far_z,
            end_ray: Ray::default(),
            view_matrix: Matrix4::default(),
            proj_matrix: Matrix4::default(),
            view_proj_matrix: Matrix4::default(),
            last_matrix: Matrix4::default(),
            inv_view_matrix: Matrix4::default(),
            inv_proj_matrix: Matrix4::default(),
            norm_view_matrix: Matrix4::default(),
        }
    }

    pub fn get_end_ray(&self) -> Ray {
        self.end_ray
    }

    pub fn get_view(&self) -> Matrix4 {
        self.view_matrix
    }

    pub fn get_proj(&self) -> Matrix4 {
        self.proj_matrix
    }

    pub fn update(&mut self, viewport: Viewport) {
        let end_dir = self.gymbal_dir * self.camera_dir;
        let arm_vector = Vector3 {
            x: 0.,
            y: 0.,
            z: self.arm_length,
        } * self.gymbal_dir;
        let end_ray = Ray {
            pos: self.base_pos + arm_vector,
            dir: end_dir,
        };
        let view_matrix = end_ray.as_matrix().inverted();
        let proj_matrix = Matrix4::project_perspective(
            self.fov,
            viewport.width as f32 / viewport.height as f32,
            self.near_z,
            self.far_z,
        );

        self.end_ray = end_ray;
        self.last_matrix = self.view_proj_matrix;
        self.view_matrix = view_matrix;
        self.proj_matrix = proj_matrix;
        self.view_proj_matrix = self.proj_matrix * self.view_matrix;
        self.inv_view_matrix = self.view_matrix.inverted();
        self.inv_proj_matrix = self.proj_matrix.inverted();
        self.norm_view_matrix = view_matrix.transform_normal();
    }
}

impl Camera for PerspectiveCamera {
    fn as_buffer(&self) -> CameraBuffer {
        CameraBuffer {
            cam_position: self.end_ray.pos,
            cam_fov_radians: self.fov,
            cam_direction: self.end_ray.dir.as_vector().as_vec4(0.),
            z_range: Vector4 {
                x: self.near_z,
                y: self.far_z,
                z: 0.,
                w: 0.,
            },
            view_matrix: self.view_matrix,
            proj_matrix: self.proj_matrix,
            view_proj_matrix: self.view_proj_matrix,
            last_matrix: self.last_matrix,
            inv_view_matrix: self.inv_view_matrix,
            inv_proj_matrix: self.inv_proj_matrix,
            norm_view_matrix: self.norm_view_matrix,
        }
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        PerspectiveCamera::new(
            Vector3::default(),
            Quaternion::default(),
            0.,
            Quaternion::default(),
            consts::FRAC_PI_4,
            0.01,
            100.,
        )
    }
}
