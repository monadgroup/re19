use crate::math::RgbColor;
use crate::texture::{Sampler, ShaderResource2D};

#[derive(Clone, Copy)]
pub struct TexturedPbrMaterial<'tex> {
    pub sampler: &'tex Sampler,
    pub albedo_map: &'tex ShaderResource2D,
    pub metallness_map: &'tex ShaderResource2D,
    pub roughness_map: &'tex ShaderResource2D,
    pub emissive_map: &'tex ShaderResource2D,
    pub normal_map: &'tex ShaderResource2D,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FlatPbrMaterial {
    pub albedo: RgbColor,
    pub metallness: f32,
    pub roughness: f32,
    pub emissive: RgbColor,
}
