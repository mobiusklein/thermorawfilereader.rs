use std::env;
use thermorawfilereader::RawFileReaderHandle;

pub fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let target = args.next().unwrap().parse::<i32>().unwrap();

    let handle = RawFileReaderHandle::open(
        path
    );
    handle.describe(target);
}
