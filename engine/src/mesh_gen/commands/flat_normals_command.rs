use super::Command;
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct FlatNormalsCommand;

impl Command for FlatNormalsCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        for face_vertices in &mut mesh.get_mut().polygons_mut() {
            // Just use the normal of the first three vertices
            let u = face_vertices[1].position - face_vertices[0].position;
            let v = face_vertices[2].position - face_vertices[0].position;
            let n = u.cross(v);
            for vertex in face_vertices {
                vertex.normal = n;
            }
        }
    }
}
