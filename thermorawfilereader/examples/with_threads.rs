use std::io;
use std::time;

use rayon::prelude::*;

use thermorawfilereader::RawFileReader;

fn main() -> io::Result<()> {
    let handle = RawFileReader::open("../tests/data/small.RAW")?;

    let start = time::Instant::now();

    let count: usize = (0..handle.len())
        .into_par_iter()
        .flat_map(|i| handle.get(i))
        .map(|s| s.data().unwrap().mz().len())
        .sum();

    let end = time::Instant::now();
    let elapsed = (end - start).as_secs_f64();
    eprintln!("{count} points in {elapsed:0.3} sec");

    Ok(())
}
