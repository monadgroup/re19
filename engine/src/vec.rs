use alloc;
use core::marker::PhantomData;
use core::{convert, mem, ops, slice};

pub struct VecIterMut<'a, T: 'a> {
    chunks: slice::Chunks<'a, u8>,
}

impl<'a, T: 'a> Iterator for VecIterMut<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {}

    fn size_hint(&self) -> (usize, Option<usize>) {}

    fn count(self) -> usize {}
}

pub struct Vec<T> {
    data: alloc::vec::Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            data: alloc::vec::Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() / mem::size_of::<T>()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn push(&mut self, val: T) {
        let bytes_slice =
            unsafe { slice::from_raw_parts(&val as *const T as *const u8, mem::size_of::<T>()) };
        self.data.extend_from_slice(bytes_slice);
        mem::forget(val);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional * mem::size_of::<T>());
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.chunks(mem::size_of::<T>()).map(|chunk| {
            let first_byte = &chunk[0];
            unsafe { &*(first_byte as *const u8 as *const T) }
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.chunks_mut(mem::size_of::<T>()).map(|chunk| {
            let first_byte = &mut chunk[0];
            unsafe { &mut *(first_byte as *mut u8 as *mut T) }
        })
    }

    pub fn first(&self) -> Option<&T> {
        self.data
            .first()
            .map(|first_byte| unsafe { &*(first_byte as *const u8 as *const T) })
    }

    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.data
            .first_mut()
            .map(|first_byte| unsafe { &mut *(first_byte as *mut u8 as *mut T) })
    }

    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let first_byte = &self.data[(self.len() - 1) * mem::size_of::<T>()];
            Some(unsafe { &*(first_byte as *const u8 as *const T) })
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let first_byte = &mut self.data[(self.len() - 1) * mem::size_of::<T>()];
            Some(unsafe { &mut *(first_byte as *mut u8 as *mut T) })
        }
    }

    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = &[T]> {
        self.data
            .chunks(chunk_size * mem::size_of::<T>())
            .map(|chunk| {
                let first_byte = &chunk[0];
                unsafe {
                    slice::from_raw_parts(
                        first_byte as *const u8 as *const T,
                        chunk.len() / mem::size_of::<T>(),
                    )
                }
            })
    }

    pub fn chunks_mut(&mut self, chunk_size: usize) -> impl Iterator<Item = &mut [T]> {
        self.data
            .chunks_mut(chunk_size * mem::size_of::<T>())
            .map(|chunk| {
                let first_byte = &mut chunk[0];
                unsafe {
                    slice::from_raw_parts_mut(
                        first_byte as *mut u8 as *mut T,
                        chunk.len() / mem::size_of::<T>(),
                    )
                }
            })
    }
}

impl<T: Clone> Vec<T> {
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.reserve(slice.len());
        for itm in slice {
            self.push(itm.clone());
        }
    }
}

impl<T, I: slice::SliceIndex<[T]>> ops::Index<I> for Vec<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &I::Output {
        ops::Index::index(&**self, index)
    }
}

impl<T, I: slice::SliceIndex<[T]>> ops::IndexMut<I> for Vec<T> {
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        ops::IndexMut::index_mut(&mut **self, index)
    }
}

impl<T> ops::Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(&self.data[0] as *const u8 as *const T, self.len()) }
    }
}

impl<T> ops::DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(&mut self.data[0] as *mut u8 as *mut T, self.len()) }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Vec<T> {
        let mut new_vec = Vec::new();
        new_vec.extend_from_slice(&self);
        new_vec
    }
}
