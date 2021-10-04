use core::marker::PhantomData;
use core::{mem, ptr, slice};
use winapi::um::d3d11::{
    ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_FLAG, D3D11_BUFFER_DESC,
    D3D11_CPU_ACCESS_FLAG, D3D11_CPU_ACCESS_WRITE, D3D11_MAP_WRITE_DISCARD,
    D3D11_RESOURCE_MISC_BUFFER_STRUCTURED, D3D11_RESOURCE_MISC_FLAG, D3D11_SUBRESOURCE_DATA,
    D3D11_USAGE, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_USAGE_IMMUTABLE,
};

pub enum InitialData<'data, T> {
    Uninitialized(usize),
    Data(&'data [T]),
}

struct BufferInternal {
    len: usize,
    ptr: *mut ID3D11Buffer,
}

impl BufferInternal {
    fn new(
        device: *mut ID3D11Device,
        usage: D3D11_USAGE,
        initial_data: InitialData<u8>,
        type_size: usize,
        bind_flags: D3D11_BIND_FLAG,
        cpu_access_flags: D3D11_CPU_ACCESS_FLAG,
        misc_flags: D3D11_RESOURCE_MISC_FLAG,
    ) -> Self {
        let (init_data, num_elems) = match initial_data {
            InitialData::Uninitialized(num_elems) => (None, num_elems),
            InitialData::Data(data) => (
                Some(D3D11_SUBRESOURCE_DATA {
                    pSysMem: data.as_ptr() as *const _,
                    SysMemPitch: 0,
                    SysMemSlicePitch: 0,
                }),
                data.len() / type_size,
            ),
        };

        // Note: ByteWidth needs to be a multiple of 16
        let real_byte_width = (type_size * num_elems) as u32;
        let byte_width = (real_byte_width + 15) / 16 * 16;
        let desc = D3D11_BUFFER_DESC {
            Usage: usage,
            ByteWidth: byte_width,
            BindFlags: bind_flags,
            CPUAccessFlags: cpu_access_flags,
            MiscFlags: misc_flags,
            StructureByteStride: type_size as u32,
        };

        let data_ptr = init_data
            .as_ref()
            .map(|data_ref| data_ref as *const _)
            .unwrap_or(ptr::null());

        let mut ptr = ptr::null_mut();
        check_err!(unsafe { (*device).CreateBuffer(&desc, data_ptr, &mut ptr) });
        BufferInternal {
            len: num_elems,
            ptr,
        }
    }
}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        unsafe {
            (*self.ptr).Release();
        }
    }
}

pub struct Buffer<T: Copy> {
    internal: BufferInternal,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    #[inline]
    pub fn new_immutable(
        device: *mut ID3D11Device,
        initial_data: &[T],
        bind_flags: D3D11_BIND_FLAG,
    ) -> Self {
        Buffer::new(
            device,
            D3D11_USAGE_IMMUTABLE,
            InitialData::Data(initial_data),
            bind_flags,
            0,
            0,
        )
    }

    #[inline]
    pub fn new_structured(
        device: *mut ID3D11Device,
        len: usize,
        bind_flags: D3D11_BIND_FLAG,
    ) -> Self {
        Buffer::new(
            device,
            D3D11_USAGE_DEFAULT,
            InitialData::Uninitialized(len),
            bind_flags,
            0,
            D3D11_RESOURCE_MISC_BUFFER_STRUCTURED,
        )
    }

    #[inline]
    pub fn new_dynamic(
        device: *mut ID3D11Device,
        initial_data: InitialData<T>,
        bind_flags: D3D11_BIND_FLAG,
    ) -> Self {
        Buffer::new(
            device,
            D3D11_USAGE_DYNAMIC,
            initial_data,
            bind_flags,
            D3D11_CPU_ACCESS_WRITE,
            0,
        )
    }

    #[inline]
    fn new(
        device: *mut ID3D11Device,
        usage: D3D11_USAGE,
        initial_data: InitialData<T>,
        bind_flags: D3D11_BIND_FLAG,
        cpu_access_flags: D3D11_CPU_ACCESS_FLAG,
        misc_flags: D3D11_RESOURCE_MISC_FLAG,
    ) -> Self {
        let data_bytes = match initial_data {
            InitialData::Uninitialized(num_elems) => InitialData::Uninitialized(num_elems),
            InitialData::Data(data) => {
                let new_slice_len = mem::size_of::<T>() * data.len();
                let new_slice_ptr = &data[0] as *const T as *const u8;
                InitialData::Data(unsafe { slice::from_raw_parts(new_slice_ptr, new_slice_len) })
            }
        };

        Buffer {
            internal: BufferInternal::new(
                device,
                usage,
                data_bytes,
                mem::size_of::<T>(),
                bind_flags,
                cpu_access_flags,
                misc_flags,
            ),
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.internal.len
    }

    pub fn byte_len(&self) -> usize {
        self.internal.len * self.stride()
    }

    pub fn stride(&self) -> usize {
        mem::size_of::<T>()
    }

    pub fn ptr(&self) -> *mut ID3D11Buffer {
        self.internal.ptr
    }

    pub fn map(&mut self, devcon: *mut ID3D11DeviceContext) -> MappedBuffer<T> {
        MappedBuffer::new(devcon, self)
    }

    pub fn do_map<R, F: FnOnce(&mut MappedBuffer<T>) -> R>(
        &mut self,
        devcon: *mut ID3D11DeviceContext,
        cb: F,
    ) -> R {
        let mut mapped = self.map(devcon);
        cb(&mut mapped)
    }

    pub fn upload(&mut self, devcon: *mut ID3D11DeviceContext, value: T) {
        self.do_map(devcon, |buffer| {
            buffer.slice_mut()[0] = value;
        });
    }
}

pub struct MappedBuffer<'buffer, T: Copy> {
    devcon: *mut ID3D11DeviceContext,
    buffer: &'buffer mut Buffer<T>,
    ptr: *mut T,
}

impl<'buffer, T: Copy> MappedBuffer<'buffer, T> {
    fn new(devcon: *mut ID3D11DeviceContext, buffer: &'buffer mut Buffer<T>) -> Self {
        let mut mapped_resource = unsafe { mem::zeroed() };
        check_err!(unsafe {
            (*devcon).Map(
                buffer.ptr() as *mut _,
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                &mut mapped_resource,
            )
        });

        MappedBuffer {
            devcon,
            buffer,
            ptr: mapped_resource.pData as *mut T,
        }
    }

    #[inline]
    pub fn slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.buffer.len()) }
    }

    #[inline]
    pub fn slice_mut(&self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.buffer.len()) }
    }
}

impl<'buffer, T: Copy> Drop for MappedBuffer<'buffer, T> {
    fn drop(&mut self) {
        unsafe { (*self.devcon).Unmap(self.buffer.ptr() as *mut _, 0) };
    }
}
