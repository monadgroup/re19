use core::{mem, slice};
use engine::math::{Vector2, Vector3, Vector4};

pub struct Stream<'bytes> {
    bytes: &'bytes [u8],
}

impl<'bytes> Stream<'bytes> {
    pub fn new(bytes: &'bytes [u8]) -> Self {
        Stream { bytes }
    }

    fn read<T: Copy + 'static>(&mut self) -> T {
        let val_size = mem::size_of::<T>();
        let val_ptr = &self.bytes[0] as *const u8 as *const T;
        let val = unsafe { *val_ptr };
        self.seek(val_size);
        val
    }

    pub fn seek(&mut self, bytes: usize) {
        self.bytes = &self.bytes[bytes..];
    }

    pub fn read_u8(&mut self) -> u8 {
        self.read()
    }

    pub fn read_u16(&mut self) -> u16 {
        self.read()
    }

    pub fn read_u32(&mut self) -> u32 {
        self.read()
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read()
    }

    pub fn read_i16(&mut self) -> i16 {
        self.read()
    }

    pub fn read_i32(&mut self) -> i32 {
        self.read()
    }

    pub fn read_f32(&mut self) -> f32 {
        self.read()
    }

    pub fn read_vector2(&mut self) -> Vector2 {
        self.read()
    }

    pub fn read_vector3(&mut self) -> Vector3 {
        self.read()
    }

    pub fn read_vector4(&mut self) -> Vector4 {
        self.read()
    }

    pub fn substream(&mut self, size: usize) -> Stream<'bytes> {
        let (substream, new_stream) = self.bytes.split_at(size);
        self.bytes = new_stream;
        Stream::new(substream)
    }

    pub fn substream_of<T: Copy + 'static>(&mut self, count: usize) -> &'bytes [T] {
        self.substream(count * mem::size_of::<T>()).interpret()
    }

    pub fn read_substream(&mut self) -> Stream<'bytes> {
        let size = self.read_u32();
        self.substream(size as usize)
    }

    pub fn interpret<T: Copy + 'static>(&self) -> &'bytes [T] {
        let val_size = mem::size_of::<T>();
        let num_vals = self.bytes.len() / val_size;
        let val_ptr = &self.bytes[0] as *const u8 as *const T;
        unsafe { slice::from_raw_parts(val_ptr, num_vals) }
    }

    pub fn as_slice(&self) -> &'bytes [u8] {
        self.bytes
    }
}
