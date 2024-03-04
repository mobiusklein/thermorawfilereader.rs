
pub(crate) mod gen;
pub(crate) mod wrap;

pub use crate::wrap::{RawFileReaderError, RawFileReaderHandle};
pub use crate::gen::schema_generated::librawfilereader as schema;
pub use dotnetrawfilereader_sys::{get_runtime, DotNetLibraryBundle};