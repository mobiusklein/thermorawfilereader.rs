//! Read Thermo RAW files using a self-hosted `dotnet` runtime that uses Thermo Fisher's `RawFileReader` library.
//!
//! The main access point is [`RawFileReader`], via the [`RawFileReader::open`].
//!
//! # Licensing
//! By using this library, you agree to the [RawFileReader License](https://github.com/mobiusklein/thermorawfilereader.rs/raw/main/librawfilereader/lib/RawFileRdr_License_Agreement_RevA.doc)
pub(crate) mod gen;
pub(crate) mod wrap;

pub use crate::wrap::{RawFileReaderError, RawFileReader, InstrumentModel, RawSpectrum, open};
#[doc = "The FlatBuffers schema used to exchange data, see [`schema.fbs`](https://github.com/mobiusklein/thermorawfilereader.rs/blob/main/schema/schema.fbs)"]
pub use crate::gen::schema_generated::librawfilereader as schema;
pub use dotnetrawfilereader_sys::{get_runtime, DotNetLibraryBundle, set_runtime_dir};