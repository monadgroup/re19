use std::ffi::CStr;
use std::fmt;
use winapi::um::d3dcommon::ID3DBlob;

pub type D3DResult<T> = Result<T, D3DError>;

pub struct D3DError {
    blob: *mut ID3DBlob,
}

impl D3DError {
    pub fn new(blob: *mut ID3DBlob) -> Self {
        D3DError { blob }
    }

    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self.blob).GetBufferPointer() as *const i8) }
    }
}

impl fmt::Debug for D3DError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_c_str())
    }
}

impl fmt::Display for D3DError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_c_str().to_str().unwrap())
    }
}

impl Drop for D3DError {
    fn drop(&mut self) {
        unsafe {
            (*self.blob).Release();
        }
    }
}
