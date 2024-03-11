//! Reader implementation for Thermo RAW files to be used with the [`mzdata`] library.
//!
//! Depends upon the [`thermorawfilereader`] crate which manages the self-hosted `dotnet`
//! runtime.
mod thermo_raw;
mod instruments;

pub use thermo_raw::{ThermoRawType, ThermoRaw, is_thermo_raw_prefix};

/// Re-exported from [`thermorawfilereader`].
pub use thermorawfilereader::set_runtime_dir;

