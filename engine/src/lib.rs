#![no_std]
#![feature(core_intrinsics)]

#[macro_use]
extern crate alloc;

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const ::winapi::ctypes::c_char
    };
}

#[macro_export]
macro_rules! check_eq {
    ($a:expr, $b:expr) => {
        let a_val = $a;
        let b_val = $b;
        if cfg!(debug_assertions) {
            assert_eq!(a_val, b_val);
        }
    };
}

#[macro_export]
macro_rules! check_ne {
    ($a:expr, $b:expr) => {
        let a_val = $a;
        let b_val = $b;
        if cfg!(debug_assertions) {
            assert_ne!(a_val, b_val);
        }
    };
}

#[macro_export]
macro_rules! check_err {
    ($s:expr) => {
        crate::check_eq!($s, 0)
    };
}

#[macro_export]
macro_rules! perf {
    ($ctx:expr, $name:expr, $call:expr) => {
        let perf = $ctx.perf.start_gpu_str($name);
        $call;
        $ctx.perf.end(perf);
    };
}

#[macro_export]
macro_rules! offset_of {
    ($t: path => $f: ident) => {
        unsafe { $crate::field_offset::FieldOffset::<$t, _>::new(|x| {
            let $t { ref $f, .. } = *x;
            $f
        }) }
    };
    ($t: path => $f: ident: $($rest: tt)*) => {
        offset_of!($t => $f) + offset_of!($($rest)*)
    };
}

pub mod animation;
pub mod binding;
pub mod blend_state;
pub mod buffer;
pub mod camera;
pub mod controller;
pub mod creation_context;
pub mod depth_state;
pub mod field_offset;
pub mod frame_context;
pub mod gbuffer;
pub mod generator;
pub mod material;
pub mod math;
pub mod mesh;
//pub mod mesh_gen;
pub mod object;
pub mod raster_state;
pub mod renderer;
pub mod resources;
pub mod shader_view;
pub mod target_view;
pub mod texture;
pub mod unordered_view;
pub mod vertex_layout;
pub mod viewport;
//pub mod vec;
