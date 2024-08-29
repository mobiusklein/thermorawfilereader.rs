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

    instrument.configurations().enumerate().for_each(|(i, c)| {
        println!("Conf {i}: {c}")
    });

    let file_descr = handle.file_description();
    if let Some(headers) = file_descr.trailer_headers() {
        println!("Trailer Names");
        headers.iter().for_each(|h| {
            println!("\t{h}");
        });
    }

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
        let dta = handle.get_baseline_noise(target as usize).unwrap();
        let noise = dta.noise();
        handle.set_centroid_spectra(true);
        let spec = handle.get(target as usize).unwrap();
        println!("{} peaks, {} noise points", spec.data().unwrap().len(), noise.len());

        // spec.data().unwrap().into_iter().for_each(|(mz, int)| {
        //     println!("{mz}\t{int}")
        // })

    }
    let end = time::Instant::now();
    println!("{:03} seconds", (end - start).as_secs_f32());

    Ok(())
}
