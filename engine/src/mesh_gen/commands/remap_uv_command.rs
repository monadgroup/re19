use super::Command;
use crate::math::Vector2;
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

// Points are in anticlockwise order, with p0 at the top left:
//
//   p0------p3
//   |       |
//   |       |
//   p1------p2

#[param_list]
#[derive(Default, Clone)]
pub struct RemapUvCommand {
    #[pragma(uv_0)]
    pub p0: Vector2,
    #[pragma(uv_1)]
    pub p1: Vector2,
    #[pragma(uv_2)]
    pub p2: Vector2,
    #[pragma(uv_3)]
    pub p3: Vector2,
}

impl Command for RemapUvCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        for vertex in &mut mesh.get_mut().vertices {
            let f = vertex.uv;
            let fi = 1. - f;
            let remapped = self.p0 * fi.x * fi.y
                + self.p1 * fi.x * f.y
                + self.p2 * f.x * f.y
                + self.p3 * f.x * fi.y;
            vertex.uv = remapped;
        }
    }
}
