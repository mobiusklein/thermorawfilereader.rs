//! This crate wraps the `librawfilereader` `dotnet` library and its associated dependencies,
//! manages the creation of a self-hosted `dotnet` runtime for them, and provides access to the
//! runtime. See [`thermorawfilereader`](../thermorawfilereader) for useful bindings.
//!
//! For regular use, call [`get_runtime`] to get a runtime handle, or [`set_runtime_dir`] to
//! pre-specify the location where runtime files need to be cached. Alternatively, set the
//! `DOTNET_RAWFILEREADER_BUNDLE_PATH` environment variable.
mod runtime;
mod buffer;

pub use crate::buffer::{RawVec, configure_allocator, rust_allocate_memory};
pub use crate::runtime::{BundleStore, DotNetLibraryBundle, get_runtime, set_runtime_dir};