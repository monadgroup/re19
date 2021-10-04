use crate::buffer::Buffer;
use crate::creation_context::CreationContext;
use crate::frame_context::FrameContext;
use crate::math::Matrix4;
use crate::mesh::{Mesh, Vertex};
use crate::object::ObjectBuffer;
use core::ptr;
use winapi::shared::dxgiformat::DXGI_FORMAT_R32_UINT;
use winapi::um::d3d11::{D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER};
use winapi::um::d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;

pub struct MeshObject {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
}

impl MeshObject {
    pub fn new(context: &mut CreationContext, mesh: &Mesh) -> Self {
        let vertex_buffer =
            Buffer::new_immutable(context.device, &mesh.vertices, D3D11_BIND_VERTEX_BUFFER);
        let index_buffer =
            Buffer::new_immutable(context.device, &mesh.indices, D3D11_BIND_INDEX_BUFFER);

        MeshObject {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn render(&mut self, context: &mut FrameContext, transform: Matrix4) {
        context
            .common
            .object_buffer
            .upload(context.devcon, ObjectBuffer::new(transform));

        unsafe {
            // bind IA
            (*context.devcon).IASetVertexBuffers(
                0,
                1,
                &self.vertex_buffer.ptr(),
                &(self.vertex_buffer.stride() as u32),
                &0,
            );
            (*context.devcon).IASetIndexBuffer(self.index_buffer.ptr(), DXGI_FORMAT_R32_UINT, 0);
            (*context.devcon).IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            // draw
            (*context.devcon).DrawIndexed(self.index_buffer.len() as u32, 0, 0);

            // unbind IA
            (*context.devcon).IASetVertexBuffers(0, 1, &ptr::null_mut(), &0, &0);
            (*context.devcon).IASetIndexBuffer(ptr::null_mut(), 0, 0);
        }
    }
}
