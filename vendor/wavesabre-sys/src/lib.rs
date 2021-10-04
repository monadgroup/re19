#![no_std]

use core::ffi::c_void;

extern "C" {
    pub fn AudioInit(
        is_prerender: u8,
        prerender_callback: Option<extern "C" fn(f64, *mut c_void)>,
        prerender_data: *mut c_void,
    );
    pub fn AudioPlay();
    pub fn AudioGetPos() -> f64;
}
