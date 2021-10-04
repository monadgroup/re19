use super::Command;
use crate::math::Vector3;
use crate::mesh_gen::connectivity::{ConnectivityInfo, VertexId};
use crate::mesh_gen::{Executor, MeshHandle, Selection};
use alloc::vec::Vec;
use tool_derive::param_list;

#[param_list]
#[derive(Default, Clone)]
pub struct SmoothNormalsCommand;

impl Command for SmoothNormalsCommand {
    fn run(&self, handle: &mut MeshHandle, _selection: &mut Selection, _executor: &dyn Executor) {
        let (mesh, connectivity) = handle.connectivity();

        // Determine the new normals for each point
        let new_normals: Vec<_> = mesh
            .vertices
            .iter()
            .enumerate()
            .map(|(vertex_index, vertex)| {
                let vertex_id = VertexId::new(vertex_index);
                let connected_edges = connectivity.get_vertex_connected_edges(vertex_id);

                let edge_count = connected_edges
                    .iter()
                    .filter(|edge| edge.get().is_some())
                    .count();
                let normal_sum = connected_edges
                    .iter()
                    .filter_map(|edge| edge.get())
                    .map(|edge| {
                        let [v_a, v_b] = ConnectivityInfo::get_edge_vertices(edge);
                        let vert_a = mesh.vertices[v_a.get()];
                        let vert_b = mesh.vertices[v_b.get()];

                        // Figure out which of the two vertices is the one we want
                        if vert_a.position.is_close_to(vertex.position) {
                            vert_a.normal
                        } else {
                            debug_assert!(vert_b.normal.is_close_to(vertex.position));

                            vert_b.normal
                        }
                    })
                    .sum::<Vector3>();
                normal_sum / edge_count as f32
            })
            .collect();

        // Apply the normals
        for (vertex, new_normal) in mesh.vertices.iter_mut().zip(new_normals.into_iter()) {
            vertex.normal = new_normal;
        }
    }
}
