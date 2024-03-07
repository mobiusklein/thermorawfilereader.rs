use std::{env, io, time};
use thermorawfilereader::RawFileReaderHandle;

pub fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let target = args.next().unwrap().parse::<i32>().unwrap();

    let mut handle = RawFileReaderHandle::open(
        path
    )?;

    let start = time::Instant::now();
    if target < 0 {
        println!("Counting MSn spectra");
        handle.set_signal_loading(false);
        let ms2_scans = handle.iter().filter(|buf| {
            buf.view().ms_level() == 2
        }).count();
        println!("Found {ms2_scans} MSn spectra");
    } else {
        handle.describe(target as usize);
    }
    let end = time::Instant::now();
    println!("{:03} seconds", (end - start).as_secs_f32());

    Ok(())
}
