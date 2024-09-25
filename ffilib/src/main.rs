
use std::ffi::c_void;

#[allow(unused)]
#[link(name = "librawfilereader", kind="static")]
#[link(name = "Runtime.ServerGC", kind="static")]
#[link(name = "eventpipe-disabled", kind="static")]
#[link(name = "System.Globalization.Native.Aot", kind="static")]
#[link(name = "System.IO.Compression.Native.Aot", kind="static")]
#[link(name = "Runtime.VxsortEnabled", kind = "static")]
#[link(name = "bootstrapperdll", kind = "static", modifiers = "+whole-archive")]
#[link(name = "bcrypt", kind="static")]
#[link(name = "ole32", kind="static")]
#[link(name = "user32", kind="static")]
extern {
    fn rawfilereader_test_add(a: i32, b: i32) -> i32;

    fn rawfilereader_open(text_ptr: *const u8, text_length: i32) -> *mut c_void;
    fn rawfilereader_close(handle: *mut c_void);

    fn rawfilereader_status(handle: *mut c_void) -> u32;
    fn rawfilereader_first_spectrum(handle: *mut c_void) -> i32;
}

mod buffer;


fn main() {
    println!("Trying to open");
    let fp = "../tests/data/small.RAW";
    unsafe {
        let value = rawfilereader_test_add(21, 32);
        println!("Summed: {value}");

        let handle = rawfilereader_open(fp.as_ptr(), fp.len() as i32);
        // rawfilereader_close(handle);
        let status = rawfilereader_status(handle);
        println!("Got status: {status}");
    };
    println!("Did open")
}
