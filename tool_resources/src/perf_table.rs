use super::cpu_perf_query::CpuPerfQuery;
use super::gpu_perf_query::GpuPerfQuery;
use std::borrow::Cow;
use std::{iter, mem, ptr};
use winapi::um::d3d11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Query, D3D11_QUERY_DATA_TIMESTAMP_DISJOINT,
    D3D11_QUERY_DESC, D3D11_QUERY_TIMESTAMP, D3D11_QUERY_TIMESTAMP_DISJOINT,
};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

#[derive(Debug, Clone)]
pub struct QueryResult<'name> {
    pub name: Cow<'name, str>,
    pub start_ms: f32,
    pub end_ms: f32,
}

pub struct QueryToken {
    query_id: usize,
}

pub struct PerfTable<'names> {
    device: *mut ID3D11Device,
    devcon: *mut ID3D11DeviceContext,

    disjoint_query: *mut ID3D11Query,
    frame_start_query: *mut ID3D11Query,
    performance_frequency: f32,
    frame_start_counter: i64,

    used_gpu_queries: usize,
    gpu_queries: Vec<GpuPerfQuery>,
    used_cpu_queries: usize,
    cpu_queries: Vec<CpuPerfQuery>,
    active_queries: Vec<(Cow<'names, str>, PerfQueryRef, bool)>,
    last_results: Vec<QueryResult<'names>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PerfQueryRef {
    Gpu(usize),
    Cpu(usize),
}

impl<'names> PerfTable<'names> {
    pub fn new(device: *mut ID3D11Device, devcon: *mut ID3D11DeviceContext) -> Self {
        let disjoint_query_desc = D3D11_QUERY_DESC {
            Query: D3D11_QUERY_TIMESTAMP_DISJOINT,
            MiscFlags: 0,
        };
        let mut disjoint_query = ptr::null_mut();
        unsafe {
            (*device).CreateQuery(&disjoint_query_desc, &mut disjoint_query);
        }

        let frame_start_query_desc = D3D11_QUERY_DESC {
            Query: D3D11_QUERY_TIMESTAMP,
            MiscFlags: 0,
        };
        let mut frame_start_query = ptr::null_mut();
        unsafe {
            (*device).CreateQuery(&frame_start_query_desc, &mut frame_start_query);
        }

        let mut performance_frequency: i64 = 0;
        unsafe {
            QueryPerformanceFrequency(&mut performance_frequency as *mut i64 as *mut _);
        }

        PerfTable {
            device,
            devcon,
            disjoint_query,
            frame_start_query,
            performance_frequency: performance_frequency as f32 / 1000.,
            frame_start_counter: 0,
            used_gpu_queries: 0,
            gpu_queries: Vec::new(),
            used_cpu_queries: 0,
            cpu_queries: Vec::new(),
            active_queries: Vec::new(),
            last_results: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.used_gpu_queries = 0;
        self.used_cpu_queries = 0;
        self.active_queries.clear();
        unsafe {
            QueryPerformanceCounter(&mut self.frame_start_counter as *mut i64 as *mut _);
        }
        unsafe {
            (*self.devcon).Begin(self.disjoint_query as *mut _);
            (*self.devcon).End(self.frame_start_query as *mut _);
        }
    }

    fn start_gpu(&mut self, name: Cow<'names, str>) -> QueryToken {
        let query_index = self.used_gpu_queries;
        self.used_gpu_queries += 1;

        let query = match self.gpu_queries.get_mut(query_index) {
            Some(cached_query) => cached_query,
            None => {
                self.gpu_queries
                    .push(GpuPerfQuery::new(self.device, self.devcon));
                self.gpu_queries.last_mut().unwrap()
            }
        };
        query.start();

        let query_id = self.active_queries.len();
        self.active_queries
            .push((name, PerfQueryRef::Gpu(query_index), true));
        QueryToken { query_id }
    }

    pub fn start_gpu_str(&mut self, name: &'names str) -> QueryToken {
        self.start_gpu(Cow::Borrowed(name))
    }

    pub fn start_gpu_string(&mut self, name: String) -> QueryToken {
        self.start_gpu(Cow::Owned(name))
    }

    fn start_cpu(&mut self, name: Cow<'names, str>) -> QueryToken {
        let query_index = self.used_cpu_queries;
        self.used_cpu_queries += 1;

        let query = match self.cpu_queries.get_mut(query_index) {
            Some(cached_query) => cached_query,
            None => {
                self.cpu_queries.push(CpuPerfQuery::new());
                self.cpu_queries.last_mut().unwrap()
            }
        };
        query.start();

        let query_id = self.active_queries.len();
        self.active_queries
            .push((name, PerfQueryRef::Cpu(query_index), true));
        QueryToken { query_id }
    }

    pub fn start_cpu_str(&mut self, name: &'names str) -> QueryToken {
        self.start_cpu(Cow::Borrowed(name))
    }

    pub fn start_cpu_string(&mut self, name: String) -> QueryToken {
        self.start_cpu(Cow::Owned(name))
    }

    pub fn end(&mut self, token: QueryToken) {
        let active_query = &mut self.active_queries[token.query_id];
        active_query.2 = false; // mark it as being ended
        match active_query.1 {
            PerfQueryRef::Gpu(gpu_query) => self.gpu_queries[gpu_query].end(),
            PerfQueryRef::Cpu(cpu_query) => self.cpu_queries[cpu_query].end(),
        }
    }

    pub fn end_frame(&mut self) {
        unsafe {
            (*self.devcon).End(self.disjoint_query as *mut _);
        }

        // Ensure there are no uncompleted queries
        for (name, _, is_running) in &self.active_queries {
            if *is_running {
                panic!("A query was not ended: {}", name);
            }
        }

        // Trim any excess queries that weren't used
        self.gpu_queries.truncate(self.used_gpu_queries);
        self.cpu_queries.truncate(self.used_cpu_queries);

        // Wait for data to be available
        loop {
            let is_available = unsafe {
                (*self.devcon).GetData(self.disjoint_query as *mut _, ptr::null_mut(), 0, 0)
            };

            if is_available != 1 {
                break;
            }
        }

        // Check whether timestamps were disjoint last frame, and return the previous results we
        // have if so.
        let mut ts_disjoint: D3D11_QUERY_DATA_TIMESTAMP_DISJOINT = unsafe { mem::zeroed() };
        unsafe {
            (*self.devcon).GetData(
                self.disjoint_query as *mut _,
                &mut ts_disjoint as *mut D3D11_QUERY_DATA_TIMESTAMP_DISJOINT as *mut _,
                mem::size_of::<D3D11_QUERY_DATA_TIMESTAMP_DISJOINT>() as u32,
                0,
            );
        }
        if ts_disjoint.Disjoint != 0 {
            return;
        }
        let gpu_ms_frequency = ts_disjoint.Frequency as f32 / 1000.;

        let mut frame_start_ticks: u64 = 0;
        unsafe {
            (*self.devcon).GetData(
                self.frame_start_query as *mut _,
                &mut frame_start_ticks as *mut u64 as *mut _,
                mem::size_of::<u64>() as u32,
                0,
            );
        }

        let gpu_queries = &self.gpu_queries;
        let cpu_queries = &self.cpu_queries;
        let frame_start_counter = self.frame_start_counter;
        let cpu_ms_frequency = self.performance_frequency;

        let last_results = self.last_results.iter().map(Some).chain(iter::repeat(None));
        let new_results: Vec<_> = self
            .active_queries
            .drain(..)
            .zip(last_results)
            .map(|((query_name, query_ref, _), last_result)| {
                let (query_start_ms, query_end_ms) = match query_ref {
                    PerfQueryRef::Gpu(gpu_query) => {
                        gpu_queries[gpu_query].retrieve(frame_start_ticks, gpu_ms_frequency)
                    }
                    PerfQueryRef::Cpu(cpu_query) => {
                        cpu_queries[cpu_query].retrieve(frame_start_counter, cpu_ms_frequency)
                    }
                };

                let (normalized_start_ms, normalized_end_ms) =
                    if let Some(last_result) = last_result {
                        if last_result.name == query_name {
                            (
                                (query_start_ms + last_result.start_ms * 9.) / 10.,
                                (query_end_ms + last_result.end_ms * 9.) / 10.,
                            )
                        } else {
                            (query_start_ms, query_end_ms)
                        }
                    } else {
                        (query_start_ms, query_end_ms)
                    };

                QueryResult {
                    name: query_name,
                    start_ms: normalized_start_ms,
                    end_ms: normalized_end_ms,
                }
            })
            .collect();
        self.last_results = new_results;
    }

    pub fn last_results(&self) -> &[QueryResult] {
        &self.last_results
    }
}

impl<'names> Drop for PerfTable<'names> {
    fn drop(&mut self) {
        unsafe {
            (*self.disjoint_query).Release();
            (*self.frame_start_query).Release();
        }
    }
}
