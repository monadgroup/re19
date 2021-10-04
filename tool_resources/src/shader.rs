use super::d3d_include::{D3DInclude, D3DIncludeDispatcher};
use super::shader_manager::DependencyGetter;
use super::shader_manager::GenericShader;
use crate::d3d_error::{D3DError, D3DResult};
use path_abs::{PathDir, PathFile};
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::{io, mem, ptr};
use winapi::shared::minwindef::{LPCVOID, UINT};
use winapi::shared::winerror::{E_FAIL, S_OK};
use winapi::um::d3d11::{
    ID3D11ComputeShader, ID3D11Device, ID3D11DomainShader, ID3D11GeometryShader, ID3D11HullShader,
    ID3D11PixelShader, ID3D11VertexShader,
};
use winapi::um::d3dcommon::{ID3DBlob, D3D_INCLUDE_TYPE};
use winapi::um::d3dcompiler::{
    D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
    D3DCOMPILE_SKIP_OPTIMIZATION,
};
use winapi::um::winnt::{HRESULT, LPCSTR};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ShaderType {
    Vertex,
    Geometry,
    Pixel,
    Hull,
    Domain,
    Compute,
}

fn get_target_name(shader_type: ShaderType) -> &'static str {
    match shader_type {
        ShaderType::Vertex => "vs_5_0\0",
        ShaderType::Geometry => "gs_5_0\0",
        ShaderType::Pixel => "ps_5_0\0",
        ShaderType::Hull => "hs_5_0\0",
        ShaderType::Domain => "ds_5_0\0",
        ShaderType::Compute => "cs_5_0\0",
    }
}

pub trait ShaderPointer {
    fn shader_type() -> ShaderType;
    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>>;
    unsafe fn release(&mut self);
    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT;
}
impl ShaderPointer for ID3D11VertexShader {
    fn shader_type() -> ShaderType {
        ShaderType::Vertex
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_vertex()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreateVertexShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}
impl ShaderPointer for ID3D11GeometryShader {
    fn shader_type() -> ShaderType {
        ShaderType::Geometry
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_geometry()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreateGeometryShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}
impl ShaderPointer for ID3D11PixelShader {
    fn shader_type() -> ShaderType {
        ShaderType::Pixel
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_pixel()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreatePixelShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}
impl ShaderPointer for ID3D11HullShader {
    fn shader_type() -> ShaderType {
        ShaderType::Hull
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_hull()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreateHullShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}
impl ShaderPointer for ID3D11DomainShader {
    fn shader_type() -> ShaderType {
        ShaderType::Domain
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_domain()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreateDomainShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}
impl ShaderPointer for ID3D11ComputeShader {
    fn shader_type() -> ShaderType {
        ShaderType::Compute
    }

    fn unwrap_generic(generic: &GenericShader) -> Option<&Shader<Self>> {
        generic.as_compute()
    }

    unsafe fn release(&mut self) {
        self.Release();
    }

    unsafe fn create_instance(
        device: *mut ID3D11Device,
        bytecode: *const c_void,
        bytecode_size: usize,
        out_ptr: *mut *mut Self,
    ) -> HRESULT {
        (*device).CreateComputeShader(bytecode, bytecode_size, ptr::null_mut(), out_ptr)
    }
}

struct ShaderResolver<'dep> {
    dependency_getter: &'dep mut DependencyGetter,
    directory_stack: Vec<PathDir>,
    visited_paths: HashSet<PathFile>,
}

impl<'dep> ShaderResolver<'dep> {
    pub fn new(base_path: PathFile, dependency_getter: &'dep mut DependencyGetter) -> Self {
        let mut visited_paths = HashSet::new();
        visited_paths.insert(base_path.clone());

        let base_directory = base_path.parent_dir().unwrap();
        ShaderResolver {
            dependency_getter,
            directory_stack: vec![base_directory],
            visited_paths,
        }
    }

    fn resolve_and_push(&mut self, file_name: &CStr) -> io::Result<String> {
        // convert / to \ in path
        let fixed_path = file_name.to_str().unwrap().replace("/", "\\");
        let full_path = {
            let base_dir = self.directory_stack.last().unwrap();
            base_dir.join(fixed_path).absolute()?.into_file()?
        };

        for _ in 0..self.directory_stack.len() {
            print!("--");
        }
        print!("{}", full_path.to_str().unwrap());

        let new_directory = full_path.parent_dir().unwrap();
        self.directory_stack.push(new_directory);

        // if we've already visited the file, just use an empty string
        if self.visited_paths.contains(&full_path) {
            println!("  X");
            return Ok("".to_string());
        }
        println!();

        self.visited_paths.insert(full_path.clone());
        self.dependency_getter.get_content(full_path)
    }

    fn pop(&mut self) {
        self.directory_stack.pop();
    }
}

impl<'dep> D3DIncludeDispatcher for ShaderResolver<'dep> {
    fn open(
        &mut self,
        _include_type: D3D_INCLUDE_TYPE,
        file_name: LPCSTR,
        _parent_data: LPCVOID,
        out_data: *mut LPCVOID,
        bytes: *mut UINT,
    ) -> HRESULT {
        let c_path = unsafe { CStr::from_ptr(file_name) };
        match self.resolve_and_push(c_path) {
            Ok(data) => {
                let c_data = CString::new(data).unwrap();
                unsafe {
                    *bytes = c_data.as_bytes().len() as u32;
                    *out_data = c_data.into_raw() as LPCVOID;
                }
                S_OK
            }
            Err(err) => {
                eprintln!(
                    "Failed to include shader {}: {}",
                    c_path.to_str().unwrap(),
                    err
                );
                E_FAIL
            }
        }
    }

    fn close(&mut self, data: LPCVOID) -> HRESULT {
        mem::drop(unsafe { CString::from_raw(data as *mut i8) });
        self.pop();
        S_OK
    }
}

struct ShaderInternal<T: ShaderPointer + ?Sized> {
    blob: *mut ID3DBlob,
    shader: *mut T,
}

impl<T: ShaderPointer> ShaderInternal<T> {
    fn new(
        device: *mut ID3D11Device,
        base_path: PathFile,
        dependency_getter: &mut DependencyGetter,
    ) -> D3DResult<Self> {
        let mut shader_blob = ptr::null_mut();
        let mut error_blob = ptr::null_mut();

        let root_content = dependency_getter.get_content(base_path.clone()).unwrap();
        let mut include_dispatcher = ShaderResolver::new(base_path.clone(), dependency_getter);
        let mut include_interface = D3DInclude::new(&mut include_dispatcher);
        println!("{}", base_path.to_str().unwrap());
        unsafe {
            D3DCompile(
                root_content.as_ptr() as LPCVOID,
                root_content.len(),
                &0,
                ptr::null(),
                include_interface.as_interface(),
                "main\0".as_ptr() as *const i8,
                get_target_name(T::shader_type()).as_ptr() as *const i8,
                if cfg!(debug_assertions) {
                    D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
                } else {
                    D3DCOMPILE_OPTIMIZATION_LEVEL3
                } | D3DCOMPILE_ENABLE_STRICTNESS,
                0,
                &mut shader_blob,
                &mut error_blob,
            );
        }

        if shader_blob.is_null() {
            Err(D3DError::new(error_blob))
        } else {
            if !error_blob.is_null() {
                let err = D3DError::new(error_blob);
                eprintln!("{}", err);
            }

            let mut shader_obj = ptr::null_mut();
            unsafe {
                T::create_instance(
                    device,
                    (*shader_blob).GetBufferPointer(),
                    (*shader_blob).GetBufferSize(),
                    &mut shader_obj,
                );
            }
            Ok(ShaderInternal::<T> {
                blob: shader_blob,
                shader: shader_obj,
            })
        }
    }
}

impl<T: ShaderPointer + ?Sized> Drop for ShaderInternal<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.blob).Release();
            (*self.shader).release();
        }
    }
}

pub struct Shader<T: ShaderPointer + ?Sized> {
    full_path: PathFile,
    internal: ShaderInternal<T>,
}

pub type VertexShader = Shader<ID3D11VertexShader>;
pub type GeometryShader = Shader<ID3D11GeometryShader>;
pub type PixelShader = Shader<ID3D11PixelShader>;
pub type HullShader = Shader<ID3D11HullShader>;
pub type DomainShader = Shader<ID3D11DomainShader>;
pub type ComputeShader = Shader<ID3D11ComputeShader>;

impl<T: ShaderPointer> Shader<T> {
    pub fn new(
        device: *mut ID3D11Device,
        full_path: PathFile,
        dependency_getter: &mut DependencyGetter,
    ) -> Self {
        let internal = ShaderInternal::new(device, full_path.clone(), dependency_getter).unwrap();
        Shader {
            full_path,
            internal,
        }
    }

    pub fn rebuild(&mut self, device: *mut ID3D11Device, dependency_getter: &mut DependencyGetter) {
        println!("Updating \"{}\"", self.full_path.to_str().unwrap());
        match ShaderInternal::new(device, self.full_path.clone(), dependency_getter) {
            Ok(shader) => self.internal = shader,
            Err(err) => {
                eprintln!("Failed to compile {}", err);
            }
        }
    }

    pub fn get_blob(&self) -> *mut ID3DBlob {
        self.internal.blob
    }

    pub fn get_shader(&self) -> *mut T {
        self.internal.shader
    }
}
