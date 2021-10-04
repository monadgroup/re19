use core::marker::PhantomData;
use core::mem;
use core::ops::Add;

pub struct FieldOffset<T, U>(usize, PhantomData<for<'a> Fn(&'a T) -> &'a U>);

impl<T, U> FieldOffset<T, U> {
    pub unsafe fn new<F: for<'a> FnOnce(&'a T) -> &'a U>(f: F) -> Self {
        let x = mem::zeroed();
        let offset = {
            let x = &x;
            let y = f(x);
            (y as *const U as usize) - (x as *const T as usize)
        };
        mem::forget(x);
        debug_assert!(offset + mem::size_of::<U>() <= mem::size_of::<T>());

        Self::new_from_offset(offset)
    }

    pub unsafe fn new_from_offset(offset: usize) -> Self {
        FieldOffset(offset, PhantomData)
    }

    pub fn apply_ptr<'a>(&self, x: *const T) -> *const U {
        ((x as usize) + self.0) as *const U
    }

    pub fn apply_ptr_mut<'a>(&self, x: *mut T) -> *mut U {
        ((x as usize) + self.0) as *mut U
    }

    pub fn apply<'a>(&self, x: &'a T) -> &'a U {
        unsafe { &*self.apply_ptr(x) }
    }

    pub fn apply_mut<'a>(&self, x: &'a mut T) -> &'a mut U {
        unsafe { &mut *self.apply_ptr_mut(x) }
    }

    pub fn get_byte_offset(&self) -> usize {
        self.0
    }

    pub unsafe fn unapply_ptr<'a>(&self, x: *const U) -> *const T {
        ((x as usize) - self.0) as *const T
    }

    pub unsafe fn unapply_ptr_mut<'a>(&self, x: *mut U) -> *mut T {
        ((x as usize) - self.0) as *mut T
    }

    pub unsafe fn unapply<'a>(&self, x: &'a U) -> &'a T {
        &*self.unapply_ptr(x)
    }

    pub unsafe fn unapply_mut<'a>(&self, x: &'a mut U) -> &'a mut T {
        &mut *self.unapply_ptr_mut(x)
    }
}

impl<T, U, V> Add<FieldOffset<U, V>> for FieldOffset<T, U> {
    type Output = FieldOffset<T, V>;

    fn add(self, other: FieldOffset<U, V>) -> FieldOffset<T, V> {
        FieldOffset(self.0 + other.0, PhantomData)
    }
}

impl<T, U> Copy for FieldOffset<T, U> {}
impl<T, U> Clone for FieldOffset<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}
