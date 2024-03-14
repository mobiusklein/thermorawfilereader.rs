use std::fs;
use std::io::{self, prelude::*};

use thermorawfilereader::RawFileReader;

fn main() -> io::Result<()> {
    let mut handle = RawFileReader::open("tests/data/small.RAW")?;
    let mut writer = io::BufWriter::new(fs::File::create("tests/data/small.mgf")?);

    handle.set_centroid_spectra(true);
    for spectrum in handle.iter().filter(|s| s.ms_level() == 2) {

        let prec = spectrum.precursor().unwrap();
        let prec_mz = prec.mz();
        // Format the charge properly by omitting it when not present
        let prec_z = match prec.charge() {
            0 => "".to_string(),
            z => {
                format!(" {z}")
            }
        };
        let prec_int = prec.intensity();

        writer.write_all(b"BEGIN IONS\n")?;
        writer.write_all(format!(r#"SCANS={}
RTINSECONDS={:0.4}
PEPMASS={prec_mz:0.4} {prec_int:0.2}{prec_z}
"#, spectrum.index() + 1, spectrum.time() * 60.0).as_bytes())?;

        if let Some(data) = spectrum.data_raw() {
            for (mz, i) in data.mz().unwrap().iter().zip(data.intensity().unwrap()) {
                writer.write_all(format!("{mz} {i}\n").as_bytes())?;
            }
        }
        writer.write_all(b"END IONS\n")?;
    }
    Ok(())
}