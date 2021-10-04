use core::marker::PhantomData;

pub struct QueryToken;

pub struct PerfTable<'names> {
    _names: PhantomData<&'names str>,
}

impl<'names> PerfTable<'names> {
    pub fn new() -> Self {
        PerfTable {
            _names: PhantomData,
        }
    }

    pub fn start_gpu_str(&mut self, _name: &'names str) -> QueryToken {
        QueryToken
    }

    pub fn end(&mut self, _token: QueryToken) {}
}
