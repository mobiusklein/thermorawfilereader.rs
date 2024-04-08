//! Read Thermo RAW files using a self-hosted .NET runtime that uses Thermo Fisher's `RawFileReader` library.
//!
//! The main access point is [`RawFileReader`], via [`RawFileReader::open`].
//!
//! # Limitations
//!
//! ## Platforms
//! `RawFileReader` requires a .NET runtime. The linking between Rust and the host's .NET runtime is managed by [`netcorehost`].
//! While it supports most major operating, you can check which versions which version of .NET supports which OS version at
//! <https://github.com/dotnet/core/blob/main/os-lifecycle-policy.md>.
//!
//! If you wish to link with a local `nethost` library instead of downloading the latest version at build time, please see
//! [`netcorehost`]'s documentation. This is still distinct from actually statically linking with .NET's `coreclr` library
//! which must be installed separately.
//!
//! ## Why no [`Read`](std::io::Read) support?
//! The underlying .NET library from Thermo's public API expects a plain file paths as strings and likes to fiddle with
//! file system locks. There is no way for it to consume .NET streams, let alone Rust analogs like [`Read`](std::io::Read),
//! so for the moment we can only open RAW files on the file system.
//!
//! # Licensing
//! By using this library, you agree to the [RawFileReader License](https://github.com/thermofisherlsms/RawFileReader/blob/main/License.doc)
mod constants;
pub(crate) mod gen;
pub(crate) mod wrap;

#[doc = "The FlatBuffers schema used to exchange data, see [`schema.fbs`](https://github.com/mobiusklein/thermorawfilereader.rs/blob/main/schema/schema.fbs)"]
pub use crate::gen::schema_generated::librawfilereader as schema;
pub use crate::wrap::{
    ChromatogramData, ChromatogramDescription, FileDescription, InstrumentConfiguration,
    InstrumentMethod, InstrumentModel, RawFileReader, RawFileReaderError, RawFileReaderIntoIter,
    RawFileReaderIter, RawSpectrum, SpectrumData,
};
pub use constants::{IonizationMode, MassAnalyzer, TraceType};
