use super::Stream;
use crate::d3d_include::{D3DInclude, D3DIncludeDispatcher};
use alloc::vec::Vec;
use core::ptr;
use engine::resources::shader::Shader;
use engine::{check_err, cstr};
use winapi::ctypes::{c_char, c_void};
use winapi::shared::minwindef::{LPCVOID, UINT};
use winapi::shared::winerror::S_OK;
use winapi::um::d3d11::ID3D11Device;
use winapi::um::d3dcommon::D3D_INCLUDE_TYPE;
use winapi::um::d3dcompiler::{D3DCompile, D3DCOMPILE_OPTIMIZATION_LEVEL3};
use winapi::um::winnt::{HRESULT, LPCSTR};

extern "C" {
    fn atoi(str: *const i8) -> i32;
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ShaderType {
    Vertex,
    Geometry,
    Pixel,
    Hull,
    Domain,
    Compute,
}

impl ShaderType {
    pub fn get_target_name(self) -> *const c_char {
        match self {
            ShaderType::Vertex => cstr!("vs_5_0"),
            ShaderType::Geometry => cstr!("gs_5_0"),
            ShaderType::Pixel => cstr!("ps_5_0"),
            ShaderType::Hull => cstr!("hs_5_0"),
            ShaderType::Domain => cstr!("ds_5_0"),
            ShaderType::Compute => cstr!("cs_5_0"),
        }
    }

    pub fn create_instance(
        self,
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
    ) -> *mut c_void {
        let mut shader_ptr = ptr::null_mut();

        let res = unsafe {
            match self {
                ShaderType::Vertex => (*device).CreateVertexShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
                ShaderType::Geometry => (*device).CreateGeometryShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
                ShaderType::Pixel => (*device).CreatePixelShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
                ShaderType::Hull => (*device).CreateHullShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
                ShaderType::Domain => (*device).CreateDomainShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
                ShaderType::Compute => (*device).CreateComputeShader(
                    bytecode,
                    bytecode_size,
                    ptr::null_mut(),
                    &mut shader_ptr as *mut *mut c_void as *mut _,
                ),
            }
        };
        check_err!(res);

        shader_ptr
    }
}

struct IncludeDispatcher<'strings> {
    strings: &'strings [&'strings [u8]],
    visited_indices: Vec<usize>,
}

impl<'strings> IncludeDispatcher<'strings> {
    fn new(strings: &'strings [&'strings [u8]], base_index: usize) -> Self {
        IncludeDispatcher {
            strings,
            visited_indices: vec![base_index],
        }
    }
}

impl<'strings> D3DIncludeDispatcher for IncludeDispatcher<'strings> {
    fn open(
        &mut self,
        _include_type: D3D_INCLUDE_TYPE,
        file_name: LPCSTR,
        _parent_data: LPCVOID,
        data: *mut LPCVOID,
        bytes: *mut UINT,
    ) -> HRESULT {
        // The filename is the index of the shader
        let shader_index = unsafe { atoi(file_name) } as usize;

        // If we've already visited it, just put in empty data
        if self
            .visited_indices
            .iter()
            .position(|&index| index == shader_index)
            .is_some()
        {
            unsafe {
                *bytes = 0;
            }
        } else {
            let str_data = self.strings[shader_index];
            unsafe {
                *bytes = str_data.len() as u32;
                *data = str_data.as_ptr() as *const c_void;
            }
        }

        self.visited_indices.push(shader_index);

        S_OK
    }

    fn close(&mut self, _data: LPCVOID) -> HRESULT {
        S_OK
    }
}

pub fn deserialize_shaders<'bytes>(
    stream: &mut Stream<'bytes>,
    device: *mut ID3D11Device,
    progress: &mut FnMut(f32),
) -> (Vec<Shader<c_void>>, &'bytes [u8]) {
    let creation_indices = stream.read_substream().interpret::<u8>();
    let entry_point_types = stream.read_substream().interpret::<ShaderType>();
    let entry_point_indices = stream.read_substream().interpret::<u8>();
    let shader_count = stream.read_u8() as usize;
    let string_lengths = stream.substream_of::<u32>(shader_count);

    let strings: Vec<_> = string_lengths
        .into_iter()
        .map(|string_length| stream.substream(*string_length as usize).as_slice())
        .collect();

    let shaders = entry_point_types
        .iter()
        .zip(entry_point_indices.iter())
        .enumerate()
        .map(|(shader_index, (entry_point_type, entry_point_index))| {
            let mut shader_blob = ptr::null_mut();
            let root_str = strings[*entry_point_index as usize];
            let mut include_dispatcher =
                IncludeDispatcher::new(&strings, *entry_point_index as usize);
            let mut include_interface = D3DInclude::new(&mut include_dispatcher);

            check_err!(unsafe {
                D3DCompile(
                    root_str.as_ptr() as *const c_void,
                    root_str.len(),
                    ptr::null(),
                    ptr::null(),
                    include_interface.as_interface(),
                    cstr!("main"),
                    entry_point_type.get_target_name(),
                    D3DCOMPILE_OPTIMIZATION_LEVEL3,
                    0,
                    &mut shader_blob,
                    ptr::null_mut(),
                )
            });

            let (blob_ptr, blob_size) = unsafe {
                (
                    (*shader_blob).GetBufferPointer(),
                    (*shader_blob).GetBufferSize(),
                )
            };
            let shader_obj = entry_point_type.create_instance(device, blob_ptr, blob_size);

            progress(shader_index as f32 / entry_point_types.len() as f32);

            Shader::new(shader_obj, shader_blob)
        })
        .collect();

    (shaders, creation_indices)
}
