use std::{ops::Deref, ptr, slice};

use netcorehost::{
    pdcstr,
    hostfxr::AssemblyDelegateLoader,
};

#[repr(C)]
pub struct RawVec<T> {
    pub(crate) data: *mut T,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
}

impl<T> RawVec<T> {
    pub fn free(&mut self) {
        if self.data != ptr::null_mut() {
            let owned = unsafe { Box::from_raw(self.data) };
            drop(owned);
            self.capacity = 0;
            self.len = 0;
            self.data = ptr::null_mut();
        }
    }
}

impl<T> Deref for RawVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}

pub extern "system" fn rust_allocate_memory(size: usize, vec: *mut RawVec<u8>) {
    eprintln!("Allocating FFI buffer of size {size}");
    let mut buf = Vec::<u8>::with_capacity(size);
    buf.resize(size, 0);
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

pub fn configure_allocator(delegate_loader: &AssemblyDelegateLoader) {
    let set_rust_allocate_memory = delegate_loader
        .get_function_with_unmanaged_callers_only::<fn(extern "system" fn(usize, *mut RawVec<u8>))>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SetRustAllocateMemory"),
        )
        .unwrap();
    set_rust_allocate_memory(rust_allocate_memory);
}