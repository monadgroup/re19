use super::Command;
use crate::math::Matrix4;
use crate::mesh::{primitives, VertexList};
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct QuadCommand {
    pub x_divisions: u32,
    pub y_divisions: u32,
    #[pragma(transform)]
    pub transform: Matrix4,
}

impl Command for QuadCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        primitives::quad(
            &mut mesh.get_mut().get_mut().transformed(self.transform),
            self.x_divisions,
            self.y_divisions,
        );
        mesh.invalidate();
    }
}
