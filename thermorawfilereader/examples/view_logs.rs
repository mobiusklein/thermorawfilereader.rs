use std::env;
use std::io;

use thermorawfilereader::RawFileReader;


pub fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap_or_else(|| panic!("Please provide a RAW file path"));
    let reader = RawFileReader::open(path)?;
    if let Some(target) = args.next() {
        if let Some(logs) = reader.get_status_logs() {
            for log in logs.bool_logs() {
                if log.name.trim() == target {
                    println!("{target} found");
                    for (t, v) in log.iter_flags() {
                        println!("{t}, {v}");
                    }
                }
            }
            for log in logs.str_logs() {
                if log.name.trim() == target {
                    println!("{target} found");
                    for (t, v) in log.iter_strings() {
                        println!("{t}, {v}");
                    }
                }
            }
            for log in logs.float_logs() {
                if log.name.trim() == target {
                    println!("{target} found");
                    for (t, v) in log.iter() {
                        println!("{t}, {v}");
                    }
                }
            }
            for log in logs.int_logs() {
                if log.name.trim() == target {
                    println!("{target} found");
                    for (t, v) in log.iter() {
                        println!("{t}, {v}");
                    }
                }
            }
        }
    } else {
        if let Some(logs) = reader.get_status_logs() {
            for log in logs.bool_logs() {
                println!(r"Log: {0}
Type: bool
Length: {1}
", log.name, log.times().len())
            }
            for log in logs.float_logs() {
                println!(r"Log: {0}
Type: float
Length: {1}
", log.name, log.times().len())
            }
            for log in logs.int_logs() {
                println!(r"Log: {0}
Type: int
Length: {1}
", log.name, log.times().len())
            }
            for log in logs.str_logs() {
                println!(r"Log: {0}
Type: string
Length: {1}
", log.name, log.times().len())
            }
        }
    }

    Ok(())
}