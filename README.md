# thermorawfilereader.rs

Read Thermo RAW files from Rust via in-process `dotnet` runtime hosting and Thermo Fisher's `RawFileReader` C# library.

## Which crate is which?

- `librawfilereader`: This is the C# library that Rust calls, it exchanges information using either opaque tokens or `FlatBuffers` (see [./schema/schema.fbs](schema/schema.fbs)). All interaction with Thermo's actual library happens in here.
- `dotnetrawfilereader-sys`: This Rust crate A) bundles the C# assemblies for `librawfilereader` and its dependencies and B) configures the loading of the `dotnet` runtime and provides it with a Rust-backed memory allocator, after a fashion.
- `thermorawfilereader`: This Rust crate provides (relatively) high level bindings for `librawfilereader` and the `FlatBuffers` messages it generates.
- `mzdata-thermo`: This Rust crate provides a (very) high level interface to `librawfilereader` via `thermorawfilereader` using the `mzdata` trait system. It may not always live here, but it makes a 1:1 comparison with mzML much more practical during development, and as a goad to fill in more metadata.
