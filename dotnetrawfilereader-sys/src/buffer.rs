use std::{mem, ops::{Deref, DerefMut}, ptr, slice};

use netcorehost::{
    pdcstr,
    hostfxr::AssemblyDelegateLoader,
};

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

impl RawVec<u8> {
    #[allow(unused)]
    fn new(size: usize) -> Self {
        let mut this = Self {
            data: ptr::null::<u8>() as *mut u8,
            len: 0,
            capacity: 0
        };

        rust_allocate_memory(size, (&mut this) as *mut RawVec<u8>);
        this
    }
}

impl<T: Clone> RawVec<T> {
    pub fn realloc(&mut self, newsize: usize) {
        let x = unsafe { mem::zeroed() };
        let buf = vec![x; newsize];
        let capacity = buf.capacity();
        let len = buf.len();
        let data = buf.leak();
        if !self.data.is_null() {
            unsafe { self.data.copy_to(data.as_mut_ptr(), self.len.min(newsize)) }
            self.free();
        }
        self.data = data.as_mut_ptr();
        self.len = len;
        self.capacity = capacity;
    }

    pub fn realloc_end(&mut self, newsize: usize) {
        if newsize <= self.len {
            return
        }
        let x = unsafe { mem::zeroed() };
        let buf = vec![x; newsize];
        let capacity = buf.capacity();
        let len = buf.len();
        let data = buf.leak();

        let offset = newsize - self.len;
        if !self.data.is_null() {
            unsafe { self.data.copy_to(data[offset..].as_mut_ptr(), self.len.min(newsize)) }
            self.free();
        }
        self.data = data.as_mut_ptr();
        self.len = len;
        self.capacity = capacity;
    }
}

/// Pretend to by a `&[T]`, a read-only view of the memory
impl<T> Deref for RawVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}

impl<T> DerefMut for RawVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        self.free()
    }
}

/// A .NET memory allocator that receives a memory address for a [`RawVec`] and
/// heap-allocates `size` that is "owned" by Rust's memory allocator and gives
/// it to .NET via the passed pointer.
pub(crate) extern "system" fn rust_allocate_memory(size: usize, vec: *mut RawVec<u8>) {
    let buf = vec![0; size];
    let capacity = buf.capacity();
    let len = buf.len();
    let data = buf.leak();
    unsafe {
        *vec = RawVec {
            data: data.as_mut_ptr(),
            len,
            capacity,
        }
    };
}


/// A .NET memory allocator that receives a memory address for a [`RawVec`] and
/// heap-allocates `size` that is "owned" by Rust's memory allocator and gives
/// it to .NET via the passed pointer.
pub(crate) extern "system" fn rust_reallocate_memory(size: usize, vec: *mut RawVec<u8>) {
    unsafe { (*vec).realloc(size) }
}

/// A .NET memory allocator that receives a memory address for a [`RawVec`] and
/// heap-allocates `size` that is "owned" by Rust's memory allocator and gives
/// it to .NET via the passed pointer.
pub(crate) extern "system" fn rust_reallocate_end_memory(size: usize, vec: *mut RawVec<u8>) {
    unsafe { (*vec).realloc_end(size) }
}


/// Configure the `dotnet` runtime to allow it to allocate unmanaged memory from Rust for
/// specific purposes. Directly depends upon the bundled `dotnet` library.
pub fn configure_allocator(delegate_loader: &AssemblyDelegateLoader) {
    let set_rust_allocate_memory = delegate_loader
        .get_function_with_unmanaged_callers_only::<fn(extern "system" fn(usize, *mut RawVec<u8>))>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SetRustAllocateMemory"),
        )
        .unwrap();
    set_rust_allocate_memory(rust_allocate_memory);

    let set_rust_allocate_memory = delegate_loader
        .get_function_with_unmanaged_callers_only::<fn(extern "system" fn(usize, *mut RawVec<u8>))>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SetRustReallocateMemory"),
        )
        .unwrap();
    set_rust_allocate_memory(rust_reallocate_memory);

    let set_rust_allocate_memory = delegate_loader
        .get_function_with_unmanaged_callers_only::<fn(extern "system" fn(usize, *mut RawVec<u8>))>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SetRustReallocateEndMemory"),
        )
        .unwrap();
    set_rust_allocate_memory(rust_reallocate_end_memory);

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_realloc() {
        let mut buf = RawVec::new(10);
        for i in 0..10 {
            buf[i] = i as u8;
        }
        buf.realloc(20);
        for i in 0..20 {
            if i < 10 {
                assert_eq!(buf[i], i as u8)
            } else {
                assert_eq!(buf[i] , 0)
            }
        }
    }

    #[test]
    fn test_realloc_end() {
        let mut buf = RawVec::new(10);
        for i in 0..10 {
            buf[i] = i as u8;
        }
        buf.realloc_end(20);
        for i in 0..20 {
            if i < 10 {
                assert_eq!(buf[i], 0)
            } else {
                assert_eq!(buf[i] , (i - 10) as u8)
            }
        }
    }
}