use core::ffi::c_void;
use core::mem;
use winapi::shared::minwindef::{LPCVOID, UINT};
use winapi::um::d3dcommon::{ID3DInclude, ID3DIncludeVtbl, D3D_INCLUDE_TYPE};
use winapi::um::winnt::{HRESULT, LPCSTR};

pub trait D3DIncludeDispatcher {
    fn open(
        &mut self,
        include_type: D3D_INCLUDE_TYPE,
        file_name: LPCSTR,
        parent_data: LPCVOID,
        data: *mut LPCVOID,
        bytes: *mut UINT,
    ) -> HRESULT;
    fn close(&mut self, data: LPCVOID) -> HRESULT;
}

#[repr(C)]
pub struct D3DInclude<'dispatcher> {
    vtable: *const ID3DIncludeVtbl,
    dispatcher: &'dispatcher mut D3DIncludeDispatcher,
}

impl<'dispatcher> D3DInclude<'dispatcher> {
    pub fn new(dispatcher: &'dispatcher mut D3DIncludeDispatcher) -> Self {
        D3DInclude {
            vtable: &INCLUDE_VTABLE,
            dispatcher,
        }
    }

    // we return a mutable reference instead of a pointer to indicate that the lifetimes
    // are attached
    pub fn as_interface(&mut self) -> &mut ID3DInclude {
        unsafe { mem::transmute(self) }
    }
}

static INCLUDE_VTABLE: ID3DIncludeVtbl = ID3DIncludeVtbl {
    Open: include_open,
    Close: include_close,
};

unsafe extern "system" fn include_open(
    this: *mut ID3DInclude,
    include_type: D3D_INCLUDE_TYPE,
    file_name: LPCSTR,
    parent_data: LPCVOID,
    data: *mut LPCVOID,
    bytes: *mut UINT,
) -> HRESULT {
    // "this" will actually be a mutable D3DInclude<> instance, which we can use to call the
    // dispatcher functions
    let reified_include = this as *mut c_void as *mut D3DInclude;
    (*reified_include)
        .dispatcher
        .open(include_type, file_name, parent_data, data, bytes)
}

unsafe extern "system" fn include_close(this: *mut ID3DInclude, data: LPCVOID) -> HRESULT {
    let reified_include = this as *mut c_void as *mut D3DInclude;
    (*reified_include).dispatcher.close(data)
}
