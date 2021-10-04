use super::Command;
use crate::math::Matrix4;
use crate::mesh::{Mesh, VertexList};
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct DuplicateCommand {
    pub x_count: u32,
    pub x_transform: Matrix4,
    pub y_count: u32,
    pub y_transform: Matrix4,
    pub z_count: u32,
    pub z_transform: Matrix4,
}

impl Command for DuplicateCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        let mut target_mesh = Mesh::new(mesh.get().polygon_sides());
        for z_index in 0..self.z_count {
            let z_amt = z_index as f32 / self.z_count as f32;

            for y_index in 0..self.y_count {
                let y_amt = y_index as f32 / self.y_count as f32;

                for x_index in 0..self.x_count {
                    let x_amt = x_index as f32 / self.x_count as f32;
                    let transform = (self.x_transform * x_amt)
                        * (self.y_transform * y_amt)
                        * (self.z_transform * z_amt);
                    mesh.get()
                        .insert_into(&mut target_mesh.get_mut().transformed(transform));
                }
            }
        }
        *mesh.get_mut() = target_mesh;
        mesh.invalidate();
    }
}
