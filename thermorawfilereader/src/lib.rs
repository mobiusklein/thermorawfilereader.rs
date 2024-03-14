//! Read Thermo RAW files using a self-hosted `dotnet` runtime that uses Thermo Fisher's `RawFileReader` library.
//!
//! The main access point is [`RawFileReader`], via [`RawFileReader::open`], or the [`open`] function, which is
//! a thin wrapper around it.
//!
//! # Limitations
//!
//! ## Platforms
//! `RawFileReader` requires a `dotnet` runtime. This is bootstrapped automatically by the [`netcorehost`] at build
//! time if the `nethost-download` feature is enabled. While it supports most major operating, you can check which
//! versions which version of `dotnet` supports which OS version at <https://github.com/dotnet/core/blob/main/os-lifecycle-policy.md>.
//!
//! If you wish to link with a local `nethost` library, please see [`netcorehost`]'s documentation.
//!
//! ## Why no [`Read`](std::io::Read) support?
//! The underlying .NET library from Thermo's public API expects a plain file path and likes to fiddle with file system
//! locks. There is no way for it to consume .NET streams, let alone Rust analogs like [`Read`](std::io::Read), so for
//! the moment we can only open RAW files on the file system.
//!
//! # Licensing
//! By using this library, you agree to the [RawFileReader License](https://github.com/thermofisherlsms/RawFileReader/blob/main/License.doc)
pub(crate) mod gen;
pub(crate) mod wrap;

pub use crate::wrap::{RawFileReaderError, RawFileReader, InstrumentModel, FileDescription, SpectrumData, RawSpectrum, open};
#[doc = "The FlatBuffers schema used to exchange data, see [`schema.fbs`](https://github.com/mobiusklein/thermorawfilereader.rs/blob/main/schema/schema.fbs)"]
pub use crate::gen::schema_generated::librawfilereader as schema;
pub use dotnetrawfilereader_sys::{get_runtime, DotNetLibraryBundle, set_runtime_dir};