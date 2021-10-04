use winapi::um::d3d11::ID3D11ShaderResourceView;

pub trait ShaderResource2D {
    fn shader_resource_ptr(&self) -> *mut ID3D11ShaderResourceView;
}
