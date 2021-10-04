use super::connectivity::{ConnectivityInfo, EdgeId, FaceId, VertexId};
use super::{Mesh, Vertex};
use crate::math::{Vector2, Vector3};
use alloc::vec::Vec;
use core::iter;

#[derive(Clone, Copy, Default)]
struct PosNormalPair(Vector3, Vector3);

impl iter::Sum for PosNormalPair {
    fn sum<I: iter::Iterator<Item = PosNormalPair>>(iter: I) -> PosNormalPair {
        let mut pair = PosNormalPair::default();
        for v in iter {
            pair.0 += v.0;
            pair.1 += v.1;
        }
        pair
    }
}

fn get_avg_vertex(vertices: &[Vertex]) -> Vertex {
    Vertex {
        position: vertices
            .iter()
            .map(|vertex| vertex.position)
            .sum::<Vector3>()
            / vertices.len() as f32,
        normal: vertices.iter().map(|vertex| vertex.normal).sum::<Vector3>()
            / vertices.len() as f32,
        uv: vertices.iter().map(|vertex| vertex.uv).sum::<Vector2>() / vertices.len() as f32,
    }
}

fn get_face_point(mesh: &Mesh, face: FaceId) -> Vertex {
    let first_vertex_index = ConnectivityInfo::get_face_first_vertex(face).get();
    let face_vertices = &mesh.vertices[first_vertex_index..(first_vertex_index + 4)];
    get_avg_vertex(face_vertices)
}

fn get_edge_point(
    mesh: &Mesh,
    connectivity: &ConnectivityInfo,
    face_points: &[Vertex],
    edge: EdgeId,
) -> Vertex {
    let edge_vertices = ConnectivityInfo::get_edge_vertices(edge);
    let edge_vertex_a = mesh.vertices[edge_vertices[0].get()];
    let edge_vertex_b = mesh.vertices[edge_vertices[1].get()];

    let face_point_a = face_points[ConnectivityInfo::get_face_containing_edge(edge).get()];
    let face_point_b = match connectivity.get_edge_adjacent_face(edge).get() {
        Some(connected_face) => face_points[connected_face.get()],
        None => face_point_a,
    };

    let avg_vertices = [edge_vertex_a, edge_vertex_b, face_point_a, face_point_b];
    get_avg_vertex(&avg_vertices)
}

pub fn catmull_clark(mesh: &mut Mesh, connectivity: &ConnectivityInfo) {
    let face_points: Vec<_> = (0..mesh.polygon_count())
        .map(|face| get_face_point(mesh, FaceId::new(face)))
        .collect();
    let edge_points: Vec<_> = (0..mesh.vertices.len())
        .map(|edge| get_edge_point(mesh, connectivity, &face_points, EdgeId::new(edge)))
        .collect();

    // Adjust the positions of the vertices in the mesh
    let adjusted_vertices: Vec<_> = mesh
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

            // Take the averages of all face points touching the vertex
            let PosNormalPair(face_pos_sum, face_normal_sum) = connected_edges
                .iter()
                .filter_map(|edge| edge.get())
                .map(|edge| ConnectivityInfo::get_face_containing_edge(edge))
                .map(|face| {
                    PosNormalPair(
                        face_points[face.get()].position,
                        face_points[face.get()].normal,
                    )
                })
                .sum::<PosNormalPair>();
            let face_pos_average = face_pos_sum / edge_count as f32;
            let face_normal_average = face_normal_sum / edge_count as f32;

            let PosNormalPair(edge_pos_sum, edge_normal_sum) = connected_edges
                .iter()
                .filter_map(|edge| edge.get())
                .map(|edge| ConnectivityInfo::get_edge_vertices(edge))
                .map(|[vertex_a, vertex_b]| {
                    PosNormalPair(
                        (mesh.vertices[vertex_a.get()].position
                            + mesh.vertices[vertex_b.get()].position)
                            / 2.,
                        (mesh.vertices[vertex_a.get()].normal
                            + mesh.vertices[vertex_b.get()].normal)
                            / 2.,
                    )
                })
                .sum::<PosNormalPair>();
            let edge_pos_average = edge_pos_sum / edge_count as f32;
            let edge_normal_average = edge_normal_sum / edge_count as f32;

            let new_pos = (face_pos_average
                + 2. * edge_pos_average
                + (edge_count as f32 - 3.) * vertex.position)
                / (edge_count as f32);
            let new_normal = (face_normal_average
                + 2. * edge_normal_average
                + (edge_count as f32 - 3.) * vertex.normal)
                / (edge_count as f32);

            Vertex {
                position: new_pos,
                normal: new_normal,
                uv: vertex.uv,
            }
        })
        .collect();

    //mesh.vertices = adjusted_vertices;

    // Create a new mesh with the subdivided faces
    let poly_count = mesh.polygon_count();
    mesh.clear();
    mesh.get_mut()
        .insert_quads_iter((0..poly_count).flat_map(|face_index| {
            let face_id = FaceId::new(face_index);
            let face_vertex = face_points[face_id.get()];

            let edge_id = ConnectivityInfo::get_face_first_edge(face_id);
            let top_left_vertex = adjusted_vertices[edge_id.get()];
            let left_edge_vertex = edge_points[edge_id.get()];

            let edge_id = ConnectivityInfo::get_next_face_edge(edge_id);
            let bottom_left_vertex = adjusted_vertices[edge_id.get()];
            let bottom_edge_vertex = edge_points[edge_id.get()];

            let edge_id = ConnectivityInfo::get_next_face_edge(edge_id);
            let bottom_right_vertex = adjusted_vertices[edge_id.get()];
            let right_edge_vertex = edge_points[edge_id.get()];

            let edge_id = ConnectivityInfo::get_next_face_edge(edge_id);
            let top_right_vertex = adjusted_vertices[edge_id.get()];
            let top_edge_vertex = edge_points[edge_id.get()];

            // Emit four quads, subdividing using the edge and face vertices
            iter::once([
                top_left_vertex,
                left_edge_vertex,
                face_vertex,
                top_edge_vertex,
            ])
            .chain(iter::once([
                top_edge_vertex,
                face_vertex,
                right_edge_vertex,
                top_right_vertex,
            ]))
            .chain(iter::once([
                left_edge_vertex,
                bottom_left_vertex,
                bottom_edge_vertex,
                face_vertex,
            ]))
            .chain(iter::once([
                face_vertex,
                bottom_edge_vertex,
                bottom_right_vertex,
                right_edge_vertex,
            ]))
        }));
}
