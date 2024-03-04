use std::{env, io};
use thermorawfilereader::RawFileReaderHandle;

pub fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let target = args.next().unwrap().parse::<i32>().unwrap();

    let handle = RawFileReaderHandle::open(
        path
    )?;

    if target < 0 {
        eprintln!("Counting MSn spectra");
        let ms2_scans = handle.iter().filter(|buf| {
            buf.view().ms_level() == 2
        }).count();
        eprintln!("Found {ms2_scans} MSn spectra");
    } else {
        handle.describe(target);
    }

    Ok(())
}
