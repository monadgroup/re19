use super::connectivity::ConnectivityInfo;
use crate::mesh::Mesh;

pub struct MeshHandle<'mesh> {
    mesh: &'mesh mut Mesh,
    connectivity: Option<ConnectivityInfo>,
}

impl<'mesh> MeshHandle<'mesh> {
    pub fn new(mesh: &'mesh mut Mesh) -> Self {
        MeshHandle {
            mesh,
            connectivity: None,
        }
    }

    pub fn get(&self) -> &Mesh {
        self.mesh
    }

    pub fn get_mut(&mut self) -> &mut Mesh {
        self.mesh
    }

    pub fn invalidate(&mut self) {
        self.connectivity = None;
    }

    pub fn connectivity(&mut self) -> (&mut Mesh, &ConnectivityInfo) {
        let mesh = self.mesh as &Mesh;
        let con_info = self
            .connectivity
            .get_or_insert_with(|| ConnectivityInfo::build(mesh));

        (self.mesh, con_info)
    }
}
