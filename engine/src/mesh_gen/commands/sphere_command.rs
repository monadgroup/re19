use super::Command;
use crate::math::Matrix4;
use crate::mesh::{primitives, VertexList};
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct SphereCommand {
    pub meridians: u32,
    pub parallels: u32,
    #[pragma(transform)]
    pub transform: Matrix4,
}

impl Command for SphereCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        primitives::sphere(
            &mut mesh.get_mut().get_mut().transformed(self.transform),
            self.meridians,
            self.parallels,
        );
        mesh.invalidate();
    }
}
