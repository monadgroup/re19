use engine::mesh::Mesh;
use engine::mesh_gen::commands::{CommandType, EditorCommand};

pub struct MeshCommand {
    pub ty: CommandType,
    pub data: Box<dyn EditorCommand>,
}

pub struct MeshDescription {
    pub name: String,
    pub is_renaming: bool,
    pub commands: Vec<MeshCommand>,
    pub selected_index: usize,
    pub current_mesh: Mesh,
    pub partial_mesh: Mesh,
}

#[derive(Default)]
pub struct MeshList {
    pub descriptions: Vec<MeshDescription>,
    pub selected_descriptions: Vec<usize>,
    pub selected_index: usize,
}

impl MeshList {
    pub fn regen_mesh(&mut self, mesh: usize) {}
}
