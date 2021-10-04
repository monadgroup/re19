use super::Command;
use crate::math::Matrix4;
use crate::mesh::{primitives, VertexList};
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct CubeCommand {
    pub x_divisions: u32,
    pub y_divisions: u32,
    #[pragma(transform)]
    pub transform: Matrix4,
}

impl Command for CubeCommand {
    fn run(&self, mesh: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        let mut a = mesh.get_mut().get_mut().transformed(self.transform);
        primitives::cube(&mut a, self.x_divisions, self.y_divisions);
        mesh.invalidate();
    }
}

// In the tool, we get a struct that looks like this:
/*pub static CUBE_COMMAND_PARAM_SCHEMA: &ParamListSchema = &ParamListSchema {
    properties: &[
        SchemaProperty {
            name: "position",
            val_type: PropertyType::Vector3,
            offset: offset_of!(...)
        },
        SchemaProperty {
            name: "scale",
            val_type: PropertyType::Vector3,
            offset: offset_of!(...)
        },
        SchemaProperty {
            name: "rotation",
            val_type: PropertyType::Rotation,
            offset: offset_of!(...)
        }
    ],

    position_pragma: Some(0),
    scale_pragma: Some(1),
    rotation_pragma: Some(2),
};*/
