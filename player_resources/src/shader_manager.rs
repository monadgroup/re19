use super::shader::Shader;
use core::marker::PhantomData;
use core::{mem, ops};
use winapi::ctypes::c_void;
use winapi::um::d3d11::{
    ID3D11ComputeShader, ID3D11Device, ID3D11DomainShader, ID3D11GeometryShader, ID3D11HullShader,
    ID3D11PixelShader, ID3D11VertexShader,
};

pub type ShaderKey<T> = (usize, PhantomData<T>);

pub type VertexKey = ShaderKey<ID3D11VertexShader>;
pub type GeometryKey = ShaderKey<ID3D11GeometryShader>;
pub type PixelKey = ShaderKey<ID3D11PixelShader>;
pub type HullKey = ShaderKey<ID3D11HullShader>;
pub type DomainKey = ShaderKey<ID3D11DomainShader>;
pub type ComputeKey = ShaderKey<ID3D11ComputeShader>;

pub struct ShaderManager<'shaders> {
    shaders: &'shaders [Shader<c_void>],
    entry_points: &'shaders [u8],
}

impl<'shaders> ShaderManager<'shaders> {
    pub fn new(shaders: &'shaders [Shader<c_void>], entry_points: &'shaders [u8]) -> Self {
        ShaderManager {
            shaders,
            entry_points,
        }
    }

    pub fn load_shader<T>(&mut self, _device: *mut ID3D11Device, _path: &str) -> ShaderKey<T> {
        let shader_index = self.entry_points[0] as usize;
        self.entry_points = &self.entry_points[1..];

        (shader_index, PhantomData)
    }

    pub fn get_shader<T>(&self, key: ShaderKey<T>) -> Option<&Shader<T>> {
        let shader_ref = unsafe { mem::transmute(&self.shaders[key.0]) };
        Some(shader_ref)
    }
}

impl<'shaders, T> ops::Index<ShaderKey<T>> for ShaderManager<'shaders> {
    type Output = Shader<T>;

    fn index(&self, key: ShaderKey<T>) -> &Shader<T> {
        self.get_shader(key).unwrap()
    }
}
