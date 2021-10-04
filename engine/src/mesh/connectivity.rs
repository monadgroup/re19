use crate::mesh::Mesh;
use alloc::vec::Vec;

// Store information on:
//  - which vertices are connected (meaning they're at the same position)
//  - which edges are connected (meaning their vertices are both connected)
//
// We make some assumptions on the format of the input data in order to simplify this task and the
// format of the output.
// Namely, we assume:
//  - All polygons have four vertices
//  - The vertices for each polygon are in anticlockwise order
//  - The vertices for each polygon are together
//
// So, a list of vertices is split up into groups of 4, with each group being one polygon.
// The output format is as follows:
//  - We assume each vertex has zero or one connected vertex. Hence vertex connectivity is stored
//    in a simple list, with each index being a vertex and the value being the index of the
//    connected vertex.
//  - A similar pattern is used for edges - each polygon has 4 edges, so we assign each edge an
//    index equal to the index of its first vertex. Then we store a connectivity list the same as
//    for vertices.

const VERTICES_PER_FACE: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexId(usize);

impl VertexId {
    pub fn new(id: usize) -> Self {
        VertexId(id)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EdgeId(usize);

impl EdgeId {
    pub fn new(id: usize) -> Self {
        EdgeId(id)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaybeEdgeId(isize);

impl MaybeEdgeId {
    fn some(val: EdgeId) -> Self {
        MaybeEdgeId(val.get() as isize)
    }

    fn none() -> Self {
        MaybeEdgeId(-1)
    }

    pub fn get(self) -> Option<EdgeId> {
        match self.0 {
            -1 => None,
            val => Some(EdgeId(val as usize)),
        }
    }
}

impl From<Option<EdgeId>> for MaybeEdgeId {
    fn from(val: Option<EdgeId>) -> Self {
        match val {
            Some(val) => MaybeEdgeId::some(val),
            None => MaybeEdgeId::none(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FaceId(usize);

impl FaceId {
    pub fn new(id: usize) -> Self {
        FaceId(id)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaybeFaceId(isize);

impl MaybeFaceId {
    fn some(val: FaceId) -> Self {
        MaybeFaceId(val.get() as isize)
    }

    fn none() -> Self {
        MaybeFaceId(-1)
    }

    pub fn get(self) -> Option<FaceId> {
        match self.0 {
            -1 => None,
            val => Some(FaceId(val as usize)),
        }
    }
}

impl From<Option<FaceId>> for MaybeFaceId {
    fn from(val: Option<FaceId>) -> Self {
        match val {
            Some(val) => MaybeFaceId::some(val),
            None => MaybeFaceId::none(),
        }
    }
}

#[derive(Clone)]
pub struct ConnectivityInfo {
    edge_connections: Vec<MaybeEdgeId>,
}

impl ConnectivityInfo {
    pub fn build(mesh: &Mesh) -> Self {
        let mut edge_connections = vec![MaybeEdgeId::none(); mesh.vertices.len()];

        for edge_index_a in 0..mesh.vertices.len() {
            // If the edge already has a connection assigned, don't bother looking for another
            if edge_connections[edge_index_a].get().is_some() {
                continue;
            }

            let edge_a_vertices = Self::get_edge_vertices(EdgeId(edge_index_a));

            // Find an edge whose points match ours
            for edge_index_b in (edge_index_a + 1)..mesh.vertices.len() {
                if edge_connections[edge_index_b].get().is_some() {
                    continue;
                }

                let edge_b_vertices = Self::get_edge_vertices(EdgeId(edge_index_b));

                // Two possible ways for the edges to match:
                //    edge_a_vertices[0] == edge_b_vertices[0] && edge_a_vertices[1] == edge_b_vertices[1]
                // or edge_a_vertices[0] == edge_b_vertices[1] && edge_a_vertices[1] == edge_b_vertices[0]
                let va0 = mesh.vertices[edge_a_vertices[0].get()].position;
                let va1 = mesh.vertices[edge_a_vertices[1].get()].position;
                let vb0 = mesh.vertices[edge_b_vertices[0].get()].position;
                let vb1 = mesh.vertices[edge_b_vertices[1].get()].position;

                // todo: do we need is_close_to or can we just use equality?
                if (va0.is_close_to(vb0) && va1.is_close_to(vb1))
                    || (va0.is_close_to(vb1) && va1.is_close_to(vb0))
                {
                    // The edge matches!
                    edge_connections[edge_index_a] = MaybeEdgeId::some(EdgeId(edge_index_b));
                    edge_connections[edge_index_b] = MaybeEdgeId::some(EdgeId(edge_index_a));
                    break;
                }
            }
        }

        ConnectivityInfo { edge_connections }
    }

    pub fn get_edge_connection(&self, edge: EdgeId) -> MaybeEdgeId {
        self.edge_connections[edge.get()]
    }

    pub fn get_edge_adjacent_face(&self, edge: EdgeId) -> MaybeFaceId {
        self.get_edge_connection(edge)
            .get()
            .map(|edge| Self::get_face_containing_edge(edge))
            .into()
    }

    pub fn get_vertex_connected_edges(&self, vertex: VertexId) -> [MaybeEdgeId; 4] {
        // Two of the edges are trivial to find: the edges that the vertex lies on.
        // But there are also two more, which belong to the faces adjacent to those two edges.
        // These are a bit harder to find, but still possible using the invariants that are outlined
        // at the top of the file.
        // Consider the case where the vertex is the top left of a rectangle. Here's a diagram:
        //
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----O~~~~~+-----+
        //      |     ~     |     |
        //      |     ~     |     |
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----+-----+-----+
        //
        // We're looking at the vertex marked "O", which is part of the two edges shown by tildes.
        // `get_edges_containing_vertex` will return the top one first and the left one second
        // (remember: anticlockwise). We can then use the connectivity info to map these to their
        // connected faces:
        //
        //      +-----+-----+-----+
        //      |     | /\  |     |
        //      |     | \/  |     |
        //      +-----O~~~~~+-----+
        //      |  /\ ~     |     |
        //      |  \/ ~     |     |
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----+-----+-----+
        //
        // These are the faces that contain the other two edges we want. In this case, we get this
        // by looking clockwise on the first edge, and anticlockwise on the second edge:
        //
        //      +-----+-----+-----+
        //      |     ~ /\  |     |
        //      |     ~ \/  |     |
        //      +~~~~~O-----+-----+
        //      |  /\ |     |     |
        //      |  \/ |     |     |
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----+-----+-----+
        //
        // As we'll see, this works when the vertex is on one of the other four positions well:
        //
        //      +-----+-----+-----+
        //      |     |   > |     |
        //      |     |   | |     |
        //      +-----+~~~~~O-----+
        //      |     |     ~-^   |
        //      |     |     ~     |
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----+-----+-----+
        //
        // Since all we're looking at is indices, these counterclockwise/clockwise rotations are
        // simply calling `get_next_face_edge` and `get_last_face_edge` respectively.
        //
        // You'll note that the method above will give edges from three different faces (remember,
        // edges that are the same but on different faces have different IDs). The two initial edges
        // will be on the 'main' face, and the other two will be on adjacent faces. But it's useful
        // to provide edges from all four faces that contact the vertex, since we can use that
        // property to find that data later along.
        //
        // This is a pretty simple operation, which we can do with just a bit of shuffling around:
        //  - The second edge is replaced with its adjacent edge
        //  - The second edge's rotation is replaced with its adjacent edge
        //
        // This gives us the following result:
        //
        //      +-----+-----+-----+
        //      |     |     ~     |
        //      |     |   ^ ~ >   |
        //      +-----+~~~~~O~~~~~+
        //      |     |   < ~ V   |
        //      |     |     ~     |
        //      +-----+-----+-----+
        //      |     |     |     |
        //      |     |     |     |
        //      +-----+-----+-----+

        let [edge_0, edge_1] = Self::get_edges_containing_vertex(vertex);

        let edge_0_adjacent = self.get_edge_connection(edge_0);
        let edge_0_rotated = edge_0_adjacent
            .get()
            .map(|edge| Self::get_last_face_edge(edge))
            .into();

        let edge_1_adjacent = self.get_edge_connection(edge_1);
        let edge_1_rotated = edge_1_adjacent
            .get()
            .map(|edge| Self::get_next_face_edge(edge));

        let flipped_edge_1_rotated = edge_1_rotated
            .and_then(|edge| self.get_edge_connection(edge).get())
            .into();

        [
            MaybeEdgeId::some(edge_0),
            edge_1_adjacent,
            flipped_edge_1_rotated,
            if edge_0_rotated == flipped_edge_1_rotated {
                MaybeEdgeId::none()
            } else {
                edge_0_rotated
            },
        ]
    }

    /*pub fn get_vertex_adjacent_faces(&self, vertex: VertexId) -> [MaybeFaceId; 4] {
        let connected_edges = self.get_vertex_connected_edges(vertex);

        // todo: make this nicer
        [
            connected_edges[0].get().map(|edge| Self::get_face_containing_edge(edge)).into(),
            connected_edges[1].get().map(|edge| Self::get_face_containing_edge(edge)).into(),
            connected_edges[2].get().map(|edge| Self::get_face_containing_edge(edge)).into(),
            connected_edges[3].get().map(|edge| Self::get_face_containing_edge(edge)).into(),
        ]
    }*/

    pub fn get_adjacent_faces(&self, face: FaceId) -> [MaybeFaceId; 4] {
        let edge = Self::get_face_first_edge(face);
        let adjacent_face_0 = self.get_edge_adjacent_face(edge);

        let edge = Self::get_next_face_edge(edge);
        let adjacent_face_1 = self.get_edge_adjacent_face(edge);

        let edge = Self::get_next_face_edge(edge);
        let adjacent_face_2 = self.get_edge_adjacent_face(edge);

        let edge = Self::get_next_face_edge(edge);
        let adjacent_face_3 = self.get_edge_adjacent_face(edge);

        [
            adjacent_face_0,
            adjacent_face_1,
            adjacent_face_2,
            adjacent_face_3,
        ]
    }

    pub fn get_edges_containing_vertex(vertex: VertexId) -> [EdgeId; 2] {
        [
            EdgeId(Self::get_last_face_vertex(vertex).get()),
            EdgeId(vertex.get()),
        ]
    }

    pub fn get_edge_vertices(edge: EdgeId) -> [VertexId; 2] {
        [
            VertexId(edge.get()),
            VertexId(Self::get_next_face_edge(edge).get()),
        ]
    }

    pub fn get_face_containing_vertex(vertex: VertexId) -> FaceId {
        FaceId(vertex.get() / VERTICES_PER_FACE)
    }

    pub fn get_face_containing_edge(edge: EdgeId) -> FaceId {
        FaceId(edge.get() / VERTICES_PER_FACE)
    }

    pub fn get_face_first_vertex(face: FaceId) -> VertexId {
        VertexId(face.get() * VERTICES_PER_FACE)
    }

    pub fn get_face_first_edge(face: FaceId) -> EdgeId {
        EdgeId(face.get() * VERTICES_PER_FACE)
    }

    pub fn get_next_face_vertex(vertex: VertexId) -> VertexId {
        let face_first_vertex_id =
            Self::get_face_first_vertex(Self::get_face_containing_vertex(vertex));
        let face_vertex = vertex.get() - face_first_vertex_id.get();
        let next_face_vertex = (face_vertex + 1) % VERTICES_PER_FACE;
        VertexId(face_first_vertex_id.get() + next_face_vertex)
    }

    pub fn get_last_face_vertex(vertex: VertexId) -> VertexId {
        let face_first_vertex_id =
            Self::get_face_first_vertex(Self::get_face_containing_vertex(vertex));
        let face_vertex = vertex.get() - face_first_vertex_id.get();
        let last_face_vertex = if face_vertex == 0 {
            VERTICES_PER_FACE - 1
        } else {
            face_vertex - 1
        };
        VertexId(face_first_vertex_id.get() + last_face_vertex)
    }

    pub fn get_next_face_edge(edge: EdgeId) -> EdgeId {
        let face_first_edge_id = Self::get_face_first_edge(Self::get_face_containing_edge(edge));
        let face_edge = edge.get() - face_first_edge_id.get();
        let next_face_edge = (face_edge + 1) % VERTICES_PER_FACE;
        EdgeId(face_first_edge_id.get() + next_face_edge)
    }

    pub fn get_last_face_edge(edge: EdgeId) -> EdgeId {
        let face_first_edge_id = Self::get_face_first_edge(Self::get_face_containing_edge(edge));
        let face_edge = edge.get() - face_first_edge_id.get();
        let last_face_edge = if face_edge == 0 {
            VERTICES_PER_FACE - 1
        } else {
            face_edge - 1
        };
        EdgeId(face_first_edge_id.get() + last_face_edge)
    }
}
