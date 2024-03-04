use std::env;

use netcorehost::{hostfxr::AssemblyDelegateLoader, pdcstr};

use thermorawfilereader::{get_runtime, RawFileReaderHandle};

#[allow(unused)]
fn test_add(delegate_loader: &AssemblyDelegateLoader) {
    let delegated = delegate_loader
        .get_function::<fn(a: i32, b: i32) -> i32>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Add"),
            pdcstr!("librawfilereader.Exports+AddFn, librawfilereader"),
        )
        .unwrap();

    let out = delegated(5, 7);
    eprintln!("out = {out}");
}

pub fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let target = args.next().unwrap().parse::<i32>().unwrap();

    let delegate_loader = get_runtime();
    let handle = RawFileReaderHandle::open(
        delegate_loader,
        path
    );
    let i = handle.len();
    handle.describe(target);
}
