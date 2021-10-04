use engine::math::{Vector2, Vector3, Vector4};
use std::{mem, slice};

pub trait Writable {
    fn write(self, buffer: &mut Vec<u8>);
}

pub trait CopyWritable: Copy + 'static {}

impl<T: CopyWritable> Writable for T {
    fn write(self, buffer: &mut Vec<u8>) {
        let val_size = mem::size_of::<T>();
        let val_ptr = &self as *const T as *const u8;
        let val_slice = unsafe { slice::from_raw_parts(val_ptr, val_size) };
        buffer.extend_from_slice(val_slice);
    }
}

impl CopyWritable for u8 {}
impl CopyWritable for u16 {}
impl CopyWritable for u32 {}
impl CopyWritable for i8 {}
impl CopyWritable for i16 {}
impl CopyWritable for i32 {}
impl CopyWritable for f32 {}
impl CopyWritable for Vector2 {}
impl CopyWritable for Vector3 {}
impl CopyWritable for Vector4 {}

pub fn write<T: Writable>(buffer: &mut Vec<u8>, val: T) {
    val.write(buffer);
}
