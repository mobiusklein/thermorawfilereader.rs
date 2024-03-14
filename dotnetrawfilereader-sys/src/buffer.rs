use std::{ops::Deref, ptr, slice};

use netcorehost::{
    pdcstr,
    hostfxr::AssemblyDelegateLoader,
};

/// A sized array for passing heap-allocated memory across the FFI memory boundary to receive
/// buffers from `dotnet`. Dereferences a `&[T]` otherwise.
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

/// A `dotnet` memory allocator that receives a memory address for a [`RawVec`] and
/// heap-allocates `size` that is "owned" by Rust's memory allocator and gives
/// it to `dotnet` via the passed pointer.
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
}