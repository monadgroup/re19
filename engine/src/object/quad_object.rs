use crate::buffer::Buffer;
use crate::creation_context::CreationContext;
use crate::math::{Vector2, Vector4};
use core::ptr;
use winapi::um::d3d11::{ID3D11DeviceContext, D3D11_BIND_VERTEX_BUFFER};
use winapi::um::d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;

pub struct QuadObject {
    vertex_buffer: Buffer<Vector2>,
}

impl QuadObject {
    pub fn new(context: &mut CreationContext, rect: Vector4) -> Self {
        let vertices = [
            Vector2 {
                x: rect.x,
                y: rect.y,
            },
            Vector2 {
                x: rect.z,
                y: rect.y,
            },
            Vector2 {
                x: rect.z,
                y: rect.w,
            },
            Vector2 {
                x: rect.x,
                y: rect.w,
            },
        ];
        let vertex_buffer = Buffer::new_immutable(
            context.device,
            &[
                vertices[3],
                vertices[1],
                vertices[0],
                vertices[3],
                vertices[2],
                vertices[1],
            ],
            D3D11_BIND_VERTEX_BUFFER,
        );

        QuadObject { vertex_buffer }
    }

    pub fn render(&mut self, devcon: *mut ID3D11DeviceContext) {
        unsafe {
            // bind IA
            (*devcon).IASetVertexBuffers(
                0,
                1,
                &self.vertex_buffer.ptr(),
                &(self.vertex_buffer.stride() as u32),
                &0,
            );
            (*devcon).IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            // draw
            (*devcon).Draw(self.vertex_buffer.len() as u32, 0);

            // unbind IA
            (*devcon).IASetInputLayout(ptr::null_mut());
            (*devcon).IASetVertexBuffers(0, 1, &ptr::null_mut(), &0, &0);
        }
    }
}
