use super::Command;
use crate::math::Matrix4;
use crate::mesh::VertexList;
use crate::mesh_gen::{Executor, ListRef, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct SublistCommand {
    pub list: ListRef,
    #[pragma(transform)]
    pub transform: Matrix4,
}

impl Command for SublistCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, executor: &dyn Executor) {
        executor
            .run(self.list)
            .insert_into(&mut mesh.get_mut().get_mut().transformed(self.transform));
        mesh.invalidate();
    }
}
