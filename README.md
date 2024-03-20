# thermorawfilereader

Read Thermo RAW files from Rust via in-process .NET runtime hosting and Thermo Fisher's `RawFileReader` .NET library. You must still install the .NET 8 runtime for this library to function. It should be compatible with .NET 7 but small changes may have to be made to the project files to function.

## Which crate is which?

- `librawfilereader`: This is the C# library that Rust calls, it exchanges information using either opaque tokens or `FlatBuffers` (see [./schema/schema.fbs](schema/schema.fbs)). All interaction with Thermo's actual library happens in here.
- `dotnetrawfilereader-sys`: This Rust crate A) bundles the C# assemblies for `librawfilereader` and its dependencies and B) configures the loading of the .NET runtime and provides it with a Rust-backed memory allocator, after a fashion.
- `thermorawfilereader`: This Rust crate provides (relatively) high level bindings for `librawfilereader` and the `FlatBuffers` messages it generates.
- `mzdata-thermo`: This Rust crate provides a (very) high level interface to `librawfilereader` via `thermorawfilereader` using the `mzdata` trait system. It may not always live here, but it makes a 1:1 comparison with mzML much more practical during development, and as a goad to fill in more metadata.

# Licenses

The code in this repository is licensed under the Apache-2.0 license, _but_ it all depends upon Thermo Fisher's `RawFileReader` library. This library has a proprietary license at [https://github.com/thermofisherlsms/RawFileReader/blob/main/License.doc](https://github.com/thermofisherlsms/RawFileReader/blob/main/License.doc), and it's assumed that you accept their license's terms by using this library.
