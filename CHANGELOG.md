# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [0.2.7] - 2024-08-30

### Added

- Add `RawFileReader::get_raw_trailers_for` to retrieve raw trailer values for a scan

### Fixed

- Fix support unicode characters in paths

## [0.2.6] - 2024-07-01

### Fixed

- Replace outdated `tempdir` dependency  with `tempfile`

## [0.2.5] - 2024-05-27

### Added

- Add resolution to `Acquisition` model

### Fixed

- Handle type coercion errors when extracting trailer values

## [0.2.4] - 2024-05-04

### Added

- Add instrument_method_count and fix instrument_method invariant

### Fixed

- Testing on ARM Mac fails. Possibly because the published Thermo libraries are not compatible with ARM-based CPUs? (#1)

## [0.2.3] - 2024-04-18

### Fixed
- The `MassAnalyzer` and `IonizationMode` Rust enums replacing the FlatBuffer enums did not completely cover the public API.
  This has been fixed.

## [0.2.2] - 2024-04-18

### Added
- Added `RawFileReader::instrument_method` to access the instrument method text segments stored in a RAW file.
  - `InstrumentMethodT` type in the FlatBuffer schema
  - `rawfilereader_instrument_method`/`Exports.InstrumentMethod` method in the .NET FFI library.
- Added `RawFileReader::tic` and `RawFileReader::bpc` to access summary chromatograms.
  - `ChromatogramDescription` in the FlatBuffer schema
  - `rawfilereader_get_tic`/`Exports.GetTIC` and `rawfilereader_get_bpc`/`Exports.GetBPC` in the .NET FFI library.
  - Added `TraceType` enum for all available trace types, but most are not yet available on the Rust side.
  - Added `IonizationMode` and `MassAnalyzer` enums to simplify decoding by downstream Rust code    since FlatBuffers enums don't support exhaustive matching.

### Changed
- The unpacking of `GetTrailerExtraInformation` now uses more efficient type coercion logic where possible.
- More wrapper types are now part of the public API.
- Increased the default buffer size for profile spectra.

## [0.2.0] - 2024-04-06

- initial release

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html

<!-- Versions -->
[unreleased]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.7...HEAD
[0.2.7]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.4
[0.2.3]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.3
[0.2.2]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.2
[0.2.1]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.1
[0.2.0]: https://github.com/mobiusklein/thermorawfilereader.rs/compare/v0.2.0