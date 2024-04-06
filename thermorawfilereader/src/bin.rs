use std::{env, io, time};
use thermorawfilereader::RawFileReader;

pub fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let target = args.next().unwrap().parse::<i32>().unwrap();

    let mut handle = RawFileReader::open(
        path
    )?;

    let instrument = handle.instrument_model();
    instrument.model().map(|s| {
        println!("Instrument Model: {}", s);
    });

    handle.bpc().data().map(|d| {
        let time = d.time();
        if !time.is_empty() {
            println!("Trace Start {} -> {} End", time.first().unwrap(), time.last().unwrap());
        }
    });

    let start = time::Instant::now();
    if target < 0 {
        handle.set_signal_loading(false);
        println!("Counting MSn spectra");
        let ms2_count = handle.iter().filter(|b| b.ms_level() > 1).count();
        println!("Found {ms2_count} MSn spectra");
        handle.set_signal_loading(true);
        handle.set_centroid_spectra(true);
        let data_points: usize = handle.iter().map(|b| {
            let view = b.view();
            let data_view = view.data().unwrap();
            data_view.mz().unwrap().len()
        }).sum();
        println!("Found {data_points} points");
    } else {
        handle.describe(target as usize);
    }
    let end = time::Instant::now();
    println!("{:03} seconds", (end - start).as_secs_f32());

    Ok(())
}
