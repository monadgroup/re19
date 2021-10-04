use std::{mem, ptr};
use winapi::um::d3d11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Query, D3D11_QUERY_DESC, D3D11_QUERY_TIMESTAMP,
};

pub struct GpuPerfQuery {
    devcon: *mut ID3D11DeviceContext,
    start_query: *mut ID3D11Query,
    end_query: *mut ID3D11Query,
}

impl GpuPerfQuery {
    pub fn new(device: *mut ID3D11Device, devcon: *mut ID3D11DeviceContext) -> Self {
        let query_desc = D3D11_QUERY_DESC {
            Query: D3D11_QUERY_TIMESTAMP,
            MiscFlags: 0,
        };

        let mut start_query = ptr::null_mut();
        unsafe {
            (*device).CreateQuery(&query_desc, &mut start_query);
        }

        let mut end_query = ptr::null_mut();
        unsafe {
            (*device).CreateQuery(&query_desc, &mut end_query);
        }

        GpuPerfQuery {
            devcon,
            start_query,
            end_query,
        }
    }

    pub fn start(&mut self) {
        unsafe {
            (*self.devcon).End(self.start_query as *mut _);
        }
    }

    pub fn end(&mut self) {
        unsafe {
            (*self.devcon).End(self.end_query as *mut _);
        }
    }

    pub fn retrieve(&self, start_ticks: u64, timer_ms_frequency: f32) -> (f32, f32) {
        let mut begin_frame = 0;
        let mut end_frame = 0;
        unsafe {
            (*self.devcon).GetData(
                self.start_query as *mut _,
                &mut begin_frame as *mut u64 as *mut _,
                mem::size_of::<u64>() as u32,
                0,
            );
            (*self.devcon).GetData(
                self.end_query as *mut _,
                &mut end_frame as *mut u64 as *mut _,
                mem::size_of::<u64>() as u32,
                0,
            );
        }

        (
            if begin_frame >= start_ticks {
                (begin_frame - start_ticks) as f32 / timer_ms_frequency
            } else {
                0.
            },
            if end_frame >= start_ticks {
                (end_frame - start_ticks) as f32 / timer_ms_frequency
            } else {
                0.
            },
        )
    }
}

impl Drop for GpuPerfQuery {
    fn drop(&mut self) {
        unsafe {
            (*self.start_query).Release();
            (*self.end_query).Release();
        }
    }
}
