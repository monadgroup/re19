use crate::mesh::Mesh;

pub type ListRef = usize;

pub trait Executor {
    fn run(&self, list: ListRef) -> &Mesh;
}
