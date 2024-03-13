//! Reader implementation for Thermo RAW files to be used with the [`mzdata`] library.
//!
//! Depends upon the [`thermorawfilereader`] crate which manages the self-hosted `dotnet`
//! runtime.
//!
//! ```rust
//! use std::io;
//!
//! use mzdata::prelude::*;
//! use mzdata_thermo::ThermoRaw;
//!
//! # fn main() -> io::Result<()> {
//! let mut reader = ThermoRaw::open_path("../tests/data/small.RAW")?;
//! let scan = reader.get_spectrum_by_index(0).unwrap();
//! assert_eq!(scan.index(), 0);
//! assert_eq!(reader.len(), 48);
//! #    Ok(())
//! # }
//! ```
//!
//! # Licensing
//! By using this library, you agree to the [RawFileReader License](https://github.com/thermofisherlsms/RawFileReader/blob/main/License.doc)
//!
mod thermo_raw;
mod instruments;

pub use thermo_raw::{ThermoRawType, ThermoRaw, is_thermo_raw_prefix};

/// Re-exported from [`thermorawfilereader`].
pub use thermorawfilereader::set_runtime_dir;

