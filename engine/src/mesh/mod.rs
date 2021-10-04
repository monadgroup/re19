use super::math::{Matrix4, Vector2, Vector3};
use alloc::vec::Vec;

//mod catmull_clark;
pub mod connectivity;
pub mod primitives;

//pub use self::catmull_clark::catmull_clark;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
}

impl Vertex {
    pub fn flip(self) -> Self {
        Vertex {
            position: self.position,
            normal: -self.normal,
            uv: self.uv,
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    polygon_sides: usize,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(polygon_sides: usize) -> Self {
        Mesh {
            polygon_sides,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn polygon_sides(&self) -> usize {
        self.polygon_sides
    }

    pub fn inverted(mut self) -> Self {
        // Flip normals of all vertices
        for vertex in &mut self.vertices {
            *vertex = vertex.flip();
        }

        // Just reverse the list of indices
        self.indices.reverse();

        self
    }

    pub fn inverted_strict(mut self) -> Self {
        // Flip normals of all vertices
        for vertex in &mut self.vertices {
            *vertex = vertex.flip();
        }

        // Reverse the indices of each polygon
        let indices_per_polygon = self.indices.len() / self.polygon_count();
        for chunk in self.indices.chunks_exact_mut(indices_per_polygon) {
            chunk.reverse();
        }

        self
    }

    pub fn polygon_count(&self) -> usize {
        self.vertices.len() / self.polygon_sides
    }

    pub fn polygons(&self) -> impl Iterator<Item = &[Vertex]> {
        self.vertices.chunks(self.polygon_sides)
    }

    pub fn polygons_mut(&mut self) -> impl Iterator<Item = &mut [Vertex]> {
        self.vertices.chunks_mut(self.polygon_sides)
    }

    pub fn insert(&mut self) -> VertexInserter {
        VertexInserter::new(self)
    }
}

pub struct VertexInserter<'mesh> {
    vertex_start_offset: u32,
    mesh: &'mesh mut Mesh,

    transform_stack: Vec<(Matrix4, Matrix4)>,
    uv_map_stack: Vec<(Vector2, Vector2)>,
}

impl<'mesh> VertexInserter<'mesh> {
    fn new(mesh: &'mesh mut Mesh) -> Self {
        VertexInserter {
            vertex_start_offset: mesh.vertices.len() as u32,
            mesh,
            transform_stack: Vec::new(),
            uv_map_stack: Vec::new(),
        }
    }

    pub fn reserve(&mut self, num_vertices: usize, num_indices: usize) {
        self.mesh.vertices.reserve(num_vertices);
        self.mesh.indices.reserve(num_indices);
    }

    fn current_transform(&self) -> (Matrix4, Matrix4) {
        self.transform_stack
            .last()
            .cloned()
            .unwrap_or((Matrix4::default(), Matrix4::default()))
    }

    pub fn push_transform(&mut self, transform: Matrix4) {
        let new_transform = self.current_transform().0 * transform;
        self.transform_stack
            .push((new_transform, new_transform.transform_normal()));
    }

    pub fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    pub fn with_transform<F: FnOnce(&mut Self)>(&mut self, transform: Matrix4, f: F) {
        self.push_transform(transform);
        f(self);
        self.pop_transform();
    }

    pub fn transformed(mut self, transform: Matrix4) -> Self {
        self.push_transform(transform);
        self
    }

    fn current_uv_map(&self) -> (Vector2, Vector2) {
        self.uv_map_stack
            .last()
            .cloned()
            .unwrap_or((Vector2 { x: 0., y: 0. }, Vector2 { x: 1., y: 1. }))
    }

    pub fn push_uv_map(&mut self, pos: Vector2, size: Vector2) {
        let (current_pos, current_size) = self.current_uv_map();
        let refined_pos = current_pos + pos * current_size;
        let refined_size = size * current_size;
        self.uv_map_stack.push((refined_pos, refined_size));
    }

    pub fn pop_uv_map(&mut self) {
        self.uv_map_stack.pop();
    }

    pub fn with_uv_map<F: FnOnce(&mut Self)>(&mut self, pos: Vector2, size: Vector2, f: F) {
        self.push_uv_map(pos, size);
        f(self);
        self.pop_uv_map();
    }

    pub fn uv_mapped(mut self, pos: Vector2, size: Vector2) -> Self {
        self.push_uv_map(pos, size);
        self
    }

    pub fn with_params<F: FnOnce(&mut Self)>(
        &mut self,
        transform: Matrix4,
        uv_pos: Vector2,
        uv_size: Vector2,
        f: F,
    ) {
        self.push_transform(transform);
        self.push_uv_map(uv_pos, uv_size);
        f(self);
        self.pop_uv_map();
        self.pop_transform();
    }

    pub fn vertex(&mut self, vertex: Vertex) {
        let (transform, normal_transform) = self.current_transform();
        let (uv_pos, uv_size) = self.current_uv_map();

        self.mesh.vertices.push(Vertex {
            position: transform * vertex.position,
            normal: normal_transform.mul_norm(vertex.normal),
            uv: uv_pos + vertex.uv * uv_size,
        });
    }

    pub fn vertices(&mut self, vertices: impl Iterator<Item = Vertex>) {
        self.mesh.vertices.reserve(vertices.size_hint().0);
        for vertex in vertices {
            self.vertex(vertex);
        }
    }

    pub fn index(&mut self, index: u32) {
        self.mesh.indices.push(self.vertex_start_offset + index);
    }

    pub fn indices(&mut self, indices: impl Iterator<Item = u32>) {
        self.mesh.indices.reserve(indices.size_hint().0);
        for index in indices {
            self.index(index);
        }
    }

    pub fn mesh(&mut self, mesh: &Mesh) {
        self.vertices(mesh.vertices.iter().cloned());
        self.indices(mesh.indices.iter().cloned());
        self.next();
    }

    pub fn quads<'s>(&'s mut self) -> QuadInserter<'s, 'mesh> {
        QuadInserter::new(self)
    }

    pub fn next(&mut self) {
        self.vertex_start_offset = self.mesh.vertices.len() as u32;
    }
}

pub struct QuadInserter<'inserter, 'mesh> {
    insert: &'inserter mut VertexInserter<'mesh>,
}

impl<'inserter, 'mesh> QuadInserter<'inserter, 'mesh> {
    fn new(insert: &'inserter mut VertexInserter<'mesh>) -> Self {
        QuadInserter { insert }
    }

    pub fn reserve(&mut self, num_quads: usize) {
        self.insert.reserve(num_quads * 4, num_quads * 6);
    }

    pub fn quad(&mut self, vertices: [Vertex; 4]) {
        self.insert.vertices(vertices.iter().cloned());
        self.insert.indices([2, 1, 0, 0, 3, 2].into_iter().cloned());
        self.insert.next();
    }

    pub fn iter(&mut self, quads: impl Iterator<Item = [Vertex; 4]>) {
        self.reserve(quads.size_hint().0);
        for vertices in quads {
            self.quad(vertices);
        }
    }
}
