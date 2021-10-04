use super::Command;
use crate::mesh::catmull_clark;
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct SubdivideCommand;

impl Command for SubdivideCommand {
    fn run(&self, handle: &mut MeshHandle, _selection: &mut Selection, executor: &dyn Executor) {
        let (mesh, connectivity) = handle.connectivity();
        catmull_clark(mesh, connectivity);
        handle.invalidate();
    }
}
