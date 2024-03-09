# librawfilereader

A C# library wrapping Thermo Fisher's [`RawFileReader`](https://github.com/thermofisherlsms/RawFileReader) library that packages data into [`FlatBuffers`](https://flatbuffers.dev/) for inexpensive unpacking across the runtime boundary. By using this library, you agree to abide by the associated license [`lib/RawFileRdr_License_Agreement_RevA.doc`](lib/RawFileRdr_License_Agreement_RevA.doc).

The schema for the exchange format is located at [../schema/schema.fbs](../schema/schema.fbs).

See [`dotnetrawfilereader-sys`](../dotnetrawfilereader-sys/) for the Rust crate that packages it for ease of distribution.