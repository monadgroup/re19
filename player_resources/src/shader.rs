use winapi::um::d3d11::{
    ID3D11ComputeShader, ID3D11DomainShader, ID3D11GeometryShader, ID3D11HullShader,
    ID3D11PixelShader, ID3D11VertexShader,
};
use winapi::um::d3dcommon::ID3DBlob;

pub type VertexShader = Shader<ID3D11VertexShader>;
pub type GeometryShader = Shader<ID3D11GeometryShader>;
pub type PixelShader = Shader<ID3D11PixelShader>;
pub type HullShader = Shader<ID3D11HullShader>;
pub type DomainShader = Shader<ID3D11DomainShader>;
pub type ComputeShader = Shader<ID3D11ComputeShader>;

pub struct Shader<T>(*mut T, *mut ID3DBlob);

impl<T> Shader<T> {
    pub fn new(shader: *mut T, blob: *mut ID3DBlob) -> Self {
        Shader(shader, blob)
    }

    pub fn get_blob(&self) -> *mut ID3DBlob {
        self.1
    }

    pub fn get_shader(&self) -> *mut T {
        self.0
    }
}
