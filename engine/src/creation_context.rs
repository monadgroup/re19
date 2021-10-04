use crate::resources::shader_manager::ShaderManager;
use crate::viewport::Viewport;
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext};

pub struct CreationContext<'manager, 'shader> {
    pub device: *mut ID3D11Device,
    pub devcon: *mut ID3D11DeviceContext,
    pub shader_manager: &'manager mut ShaderManager<'shader>,
    pub viewport: Viewport,
}
