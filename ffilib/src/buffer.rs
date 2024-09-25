use std::{ops::Deref, ptr, slice};


/// A sized array for passing heap-allocated memory across the FFI memory boundary to receive
/// buffers from .NET. Dereferences a `&[T]` otherwise.
///
/// # Safety
/// This type makes heavy use of `unsafe` operations for manual memory management. Take care
/// when using it as more than an opaque buffer. When it is dropped the memory is freed.
#[repr(C)]
pub struct RawVec<T> {
    pub(crate) data: *mut T,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
}

unsafe impl<T> Send for RawVec<T> {}

unsafe impl<T> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    /// Releases the owned memory and puts this struct into an unusable, empty
    /// state. This method is called on `drop`, but it is safe to call repeatedly.
    pub fn free(&mut self) {
        if !self.data.is_null() {
            let owned = unsafe { Box::from_raw(self.data) };
            drop(owned);
            self.capacity = 0;
            self.len = 0;
            self.data = ptr::null_mut();
        }
    }
}

/// Pretend to by a `&[T]`, a read-only view of the memory
impl<T> Deref for RawVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        self.free()
    }
}

