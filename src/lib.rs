
pub(crate) mod gen;
pub(crate) mod buffer;
pub(crate) mod wrap;
pub(crate) mod runtime;

pub use crate::wrap::{RawFileReaderError, RawFileReaderHandle, load_runtime};
pub use crate::gen::schema_generated::librawfilereader as schema;
pub use crate::buffer::{RawVec, configure_allocator};