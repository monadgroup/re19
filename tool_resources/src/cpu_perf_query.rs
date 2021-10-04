use winapi::um::profileapi::QueryPerformanceCounter;

pub struct CpuPerfQuery {
    start_time: i64,
    end_time: i64,
}

impl CpuPerfQuery {
    pub fn new() -> Self {
        CpuPerfQuery {
            start_time: 0,
            end_time: 0,
        }
    }

    pub fn start(&mut self) {
        unsafe {
            QueryPerformanceCounter(&mut self.start_time as *mut i64 as *mut _);
        }
    }

    pub fn end(&mut self) {
        unsafe {
            QueryPerformanceCounter(&mut self.end_time as *mut i64 as *mut _);
        }
    }

    pub fn retrieve(&self, start_ticks: i64, timer_ms_frequency: f32) -> (f32, f32) {
        (
            (self.start_time - start_ticks) as f32 / timer_ms_frequency,
            (self.end_time - start_ticks) as f32 / timer_ms_frequency,
        )
    }
}
