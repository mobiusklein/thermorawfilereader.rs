use std::borrow::Cow;
use std::ffi::c_void;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, ptr};

use netcorehost::hostfxr::ManagedFunction;
use netcorehost::{hostfxr::AssemblyDelegateLoader, pdcstr};

use flatbuffers::{root, Vector};

use dotnetrawfilereader_sys::RawVec;

use crate::get_runtime;
use crate::schema::{
    root_as_spectrum_description, root_as_spectrum_description_unchecked, AcquisitionT, FileDescriptionT, InstrumentConfigurationT, InstrumentModelT, Polarity, PrecursorT, SpectrumData as SpectrumDataT, SpectrumDescription, SpectrumMode
};


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A set of error codes to describe how creating and using a `RawFileReader` might fail (or not).
pub enum RawFileReaderError {
    /// No problem, move along
    Ok = 0,
    /// The file path given doesn't exist
    FileNotFound,
    /// The file path given does exist, but it's not a Thermo RAW file
    InvalidFormat,
    /// The handle provided doesn't exist, someone is doing something odd like making a new [`RawFileReader`]
    /// somehow other than [`RawFileReader::open`]
    HandleNotFound,
    /// Some other error occurred
    Error = 999,
}

impl Display for RawFileReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RawFileReaderError {}

impl From<u32> for RawFileReaderError {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::FileNotFound,
            2 => Self::InvalidFormat,
            _ => Self::Error,
        }
    }
}

#[derive()]
/// A wrapper around the `SpectrumDescription` FlatBuffer schema. It mirrors the data
/// stored there-in.
pub struct RawSpectrum {
    data: RawVec<u8>,
}

impl Debug for RawSpectrum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawSpectrum")
            .field("data-size", &self.data.len())
            .finish()
    }
}

/// A sub-set of a [`RawSpectrum`] corresponding to the m/z and intensity arrays
/// of a mass spectrum. All data is borrowed internally from the [`RawSpectrum`]'s
/// buffer.
#[derive(Debug)]
pub struct SpectrumData<'a> {
    mz: Vector<'a, f64>,
    intensity: Vector<'a, f32>
}


impl<'a> SpectrumData<'a> {

    /// The m/z array of the spectrum
    pub fn mz(&self) -> Cow<'a, [f64]> {
        #[cfg(target_endian = "big")]
        return Cow::Owned(self.mz.iter().copied().collect());
        #[cfg(target_endian = "little")]
        Cow::Borrowed(bytemuck::cast_slice(self.mz.bytes()))
    }

    /// The intensity array of the spectrum
    pub fn intensity(&self) -> Cow<'a, [f32]> {
        #[cfg(target_endian = "big")]
        return Cow::Owned(self.intensity.iter().copied().collect());
        #[cfg(target_endian = "little")]
        Cow::Borrowed(bytemuck::cast_slice(self.intensity.bytes()))
    }
}

impl RawSpectrum {
    /// Create a new [`RawSpectrum`] by wrapping an owning memory buffer
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `SpectrumDescription`
    pub fn check(&self) -> bool {
        root_as_spectrum_description(&self.data).is_ok()
    }

    /// View the underlying buffer as a `SpectrumDescription`
    pub fn view(&self) -> SpectrumDescription {
        unsafe { root_as_spectrum_description_unchecked(&self.data) }
    }

    /// Generate the "native ID" string format for the spectrum
    pub fn native_id(&self) -> String {
        format!(
            "controllerType=0 controllerNumber=1 scan={}",
            self.index() + 1
        )
    }

    /// The 0-base index of the spectrum
    pub fn index(&self) -> usize {
        self.view().index() as usize
    }

    /// The MS exponentiation level
    pub fn ms_level(&self) -> u8 {
        self.view().ms_level()
    }

    /// The scan start time, in minutes
    pub fn time(&self) -> f64 {
        self.view().time()
    }

    /// Whether the spectrum is positive or negative mode.
    /// [`Polarity`] is a FlatBuffer enum
    pub fn polarity(&self) -> Polarity {
        self.view().polarity()
    }

    /// Whether the spectrum is profile or centroid.
    /// [`SpectrumMode`] is a FlatBuffer enum
    pub fn mode(&self) -> SpectrumMode {
        self.view().mode()
    }

    pub fn precursor(&self) -> Option<&PrecursorT> {
        self.view().precursor()
    }

    /// Retrieve a view of the spectrum array data in a raw
    /// [`SpectrumDataT`] FlatBuffer struct, which is less expressive
    /// than [`SpectrumData`].
    ///
    /// This will be absent if signal loading is disabled with [`RawFileReader::set_signal_loading`].
    /// This returns a raw FlatBuffer struct. See the [schema](https://github.com/mobiusklein/thermorawfilereader.rs/blob/main/schema/schema.fbs) for more details
    pub fn data_raw(&self) -> Option<SpectrumDataT<'_>> {
        self.view().data()
    }

    /// Retrieve a view of the spectrum array data as a [`SpectrumData`],
    /// which will try to decode the m/z and intensity vectors and expose
    /// them as Rust-interpretable slices.
    ///
    /// This will be absent if signal loading is disabled with [`RawFileReader::set_signal_loading`].
    pub fn data(&self) -> Option<SpectrumData<'_>> {
        if let Some(d) = self.view().data() {
            if let Some(m) = d.mz() {
                if let Some(i) = d.intensity() {
                    return Some(SpectrumData {mz: m, intensity: i})
                }
            }
        }
        return None
    }

    /// Get the spectrum's filter string as a raw string. It's format is controlled
    /// by Thermo and some information has already been extracted into other fields.
    /// Some usecases still require parsing it directly as a string.
    pub fn filter_string(&self) -> Option<&str> {
        self.view().filter_string()
    }

    /// Get less essential information about *how* the spectrum was acquired.
    ///
    /// This is a raw FlatBuffer struct. See the [schema](https://github.com/mobiusklein/thermorawfilereader.rs/blob/main/schema/schema.fbs) for more details.
    pub fn acquisition(&self) -> Option<AcquisitionT> {
        self.view().acquisition()
    }
}


/// A wrapper around the `InstrumentModel` FlatBuffer schema. It mirrors the data
/// stored there-in.
pub struct InstrumentModel {
    data: RawVec<u8>,
}

impl InstrumentModel {
    /// Create a new [`InstrumentModel`] by wrapping an owning memory buffer
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `InstrumentModelT`
    pub fn check(&self) -> bool {
        root::<InstrumentModelT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `InstrumentModelT`
    pub fn view(&self) -> InstrumentModelT {
        root::<InstrumentModelT>(&self.data).unwrap()
    }

    /// The instrument model's name as a string.
    ///
    /// The majority of instrument models map onto the PSI-MS
    /// controlled vocabulary uniquely, but not all do.
    pub fn model(&self) -> Option<&str> {
        self.view().model()
    }

    /// The instrument's name as configured by the
    /// operator.
    pub fn name(&self) -> Option<&str> {
        self.view().name()
    }

    /// The instrument's serial number as specified by
    /// the manufacturer.
    pub fn serial_number(&self) -> Option<&str> {
        self.view().serial_number()
    }

    /// The hardware version string.
    pub fn hardware_version(&self) -> Option<&str> {
        self.view().hardware_version()
    }

    /// The acquisition software version string.
    ///
    /// This corresponds to the version of Xcalibur.
    pub fn software_version(&self) -> Option<&str> {
        self.view().software_version()
    }

    ///
    pub fn configurations(&self) -> Option<flatbuffers::Vector<'_, InstrumentConfigurationT>> {
        self.view().configurations()
    }
}

pub struct FileDescription {
    data: RawVec<u8>
}

impl FileDescription {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `FileDescriptionT`
    pub fn check(&self) -> bool {
        root::<FileDescriptionT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `FileDescriptionT`
    pub fn view(&self) -> FileDescriptionT {
        root::<FileDescriptionT>(&self.data).unwrap()
    }

    pub fn sample_id(&self) -> Option<&str> {
        self.view().sample_id()
    }

    pub fn source_file(&self) -> Option<&str> {
        self.view().source_file()
    }

    pub fn creation_date(&self) -> Option<&str> {
        self.view().creation_date()
    }

    pub fn spectra_per_ms_level(&self) -> Option<flatbuffers::Vector<'_, u32>> {
        self.view().spectra_per_ms_level()
    }
}

/// A wrapper around a `dotnet` `RawFileReader` instance. It carries a reference to a
/// `dotnet` runtime and a FFI pointer to access data through. The dotnet runtime is
/// controlled via locks and is expected to be thread-safe.
///
/// This object's lifetime controls a shared resource in the `dotnet` runtime.
pub struct RawFileReader {
    /// The token controlling the `RawFileReader` this object references
    raw_file_reader: *mut c_void,
    /// A reference to the `dotnet` runtime
    context: Arc<AssemblyDelegateLoader>,
    /// A cache for the number of spectra in the RAW file
    size: usize,
    /// A FFI function pointer to get spectra through.
    vget: ManagedFunction<extern "system" fn(*mut c_void, i32) -> RawVec<u8>>,
}

unsafe impl Send for RawFileReader {}
unsafe impl Sync for RawFileReader {}

impl Drop for RawFileReader {
    fn drop(&mut self) {
        self.close()
    }
}

impl Debug for RawFileReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawFileReader")
            .field("raw_file_reader", &self.raw_file_reader)
            .field("context", &"?")
            .field("size", &self.size)
            .finish()
    }
}

/// Open a ThermoFisher RAW file from a path. This may also create the `dotnet` runtime
/// if this is the first time it was called.
///
/// This is a wrapper around [`RawFileReader::open`]
pub fn open<P: Into<PathBuf>>(path: P) -> io::Result<RawFileReader> {
    RawFileReader::open(path)
}

impl RawFileReader {
    /// Open a ThermoFisher RAW file from a path. This may also create the `dotnet` runtime
    /// if this is the first time it was called.
    pub fn open<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let context = get_runtime();
        let open_fn = context.get_function_with_unmanaged_callers_only::<fn(text_ptr: *const u8, text_length: i32) -> *mut c_void>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Open")
        ).unwrap();
        let path: PathBuf = path.into();
        let path = path.to_string_lossy().to_string();
        let raw_file_reader = open_fn(path.as_ptr(), path.len() as i32);

        let buffer_fn = context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SpectrumDescriptionFor"),
            )
            .unwrap();

        let mut handle = Self {
            raw_file_reader,
            context,
            size: 0,
            vget: buffer_fn,
        };

        match &handle.status() {
            RawFileReaderError::Ok => {}
            RawFileReaderError::FileNotFound => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
            }
            RawFileReaderError::InvalidFormat => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "File does not appear to be a valid RAW file",
                ))
            }
            RawFileReaderError::Error | RawFileReaderError::HandleNotFound => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "An unknown error occured",
                ))
            }
        }

        handle.size = handle._impl_len();

        Ok(handle)
    }

    /// Get the scan number of the first spectrum
    pub fn first_spectrum(&self) -> i32 {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> i32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("FirstSpectrum"),
            )
            .unwrap();
        index_fn(self.raw_file_reader)
    }

    /// Get the scan number of the last spectrum
    pub fn last_spectrum(&self) -> i32 {
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> i32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("LastSpectrum"),
            )
            .unwrap();
        index_fn(self.raw_file_reader)
    }

    /// Get whether or not to retrieve the spectrum signal data when retrieving spectra
    pub fn get_signal_loading(&self) -> bool {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> u32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetSignalLoading"),
            )
            .unwrap();
        index_fn(self.raw_file_reader) > 0
    }

    /// Set whether or not to retrieve the spectrum signal data when retrieving spectra
    pub fn set_signal_loading(&mut self, value: bool) {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, u32)>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SetSignalLoading"),
            )
            .unwrap();
        index_fn(self.raw_file_reader, value as u32)
    }

    /// Get whether or not to centroid spectra if they were stored in profile mode
    pub fn get_centroid_spectra(&self) -> bool {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> u32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetCentroidSpectra"),
            )
            .unwrap();
        index_fn(self.raw_file_reader) > 0
    }

    /// Set whether or not to centroid spectra if they were stored in profile mode
    pub fn set_centroid_spectra(&mut self, value: bool) {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, u32)>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SetCentroidSpectra"),
            )
            .unwrap();
        index_fn(self.raw_file_reader, value as u32)
    }

    /// Get a [`InstrumentModel`] message describing the instrument configuration used
    /// to acquire the RAW file.
    pub fn instrument_model(&self) -> InstrumentModel {
        self.validate_impl();
        let instrument_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("InstrumentModel"),
            )
            .unwrap();
        let buf = instrument_fn(self.raw_file_reader);
        root::<InstrumentModelT>(&buf).unwrap();
        InstrumentModel::new(buf)
    }

    pub fn file_description(&self) -> FileDescription {
        self.validate_impl();
        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("FileDescription"),
            )
            .unwrap();
        let buf = descr_fn(self.raw_file_reader);
        root::<FileDescriptionT>(&buf).unwrap();
        FileDescription::new(buf)
    }

    #[inline]
    fn validate_impl(&self) {
        if self.raw_file_reader.is_null() {
            panic!("Internal handle already closed.")
        }
    }

    fn _impl_len(&self) -> usize {
        self.validate_impl();
        let index_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> i32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SpectrumCount"),
            )
            .unwrap();
        index_fn(self.raw_file_reader) as usize
    }

    #[inline]
    /// Get the number of spectra in the RAW file
    pub fn len(&self) -> usize {
        if self.size != 0 {
            self.size
        } else {
            self._impl_len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Close the RAW file, releasing resources held by the `dotnet`
    /// runtime. This places the object in an unusable state.
    ///
    /// This method is called on `drop`.
    fn close(&mut self) {
        if !self.raw_file_reader.is_null() {
            let close_fn = self
                .context
                .get_function_with_unmanaged_callers_only::<fn(*mut c_void)>(
                    pdcstr!("librawfilereader.Exports, librawfilereader"),
                    pdcstr!("Close"),
                )
                .unwrap();
            close_fn(self.raw_file_reader);
            self.raw_file_reader = ptr::null_mut();
        }
    }

    /// Get the spectrum at index `index`
    ///
    /// **Note**: The index of a spectrum is one less than its scan number
    pub fn get(&self, index: usize) -> Option<RawSpectrum> {
        if index >= self.len() {
            return None;
        }
        self.validate_impl();
        let buffer_fn = &self.vget;
        let buffer = buffer_fn(self.raw_file_reader, (index as i32) + 1);
        Some(RawSpectrum::new(buffer))
    }

    /// A utility for debugging, get a spectrum and access some of its fields, printing them
    /// to `STDOUT`
    pub fn describe(&self, index: usize) {
        if let Some(buf) = self.get(index) {
            let descr = buf.view();
            println!(
                "{}|{:?} -> {:?} | has data? {}",
                descr.index(),
                descr.polarity(),
                descr.precursor(),
                descr.data().is_some()
            );
            println!("Filter: {}", descr.filter_string().unwrap());
            let acq = descr.acquisition().unwrap();
            println!(
                "{:?} {:?} {}-{}",
                acq.mass_analyzer(),
                acq.ionization_mode(),
                acq.low_mz(),
                acq.high_mz()
            );
            if let Some(dat) = descr.data() {
                let intens_opt = dat.intensity();
                let intens = intens_opt.as_ref().unwrap();
                let val = intens.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
                println!(
                    "Received {} data points, base peak intensity {}",
                    intens.len(),
                    val
                );
            }
        } else {
            println!("Spectrum at {index}/{} not found", self.len())
        }
    }

    /// Create an iterator over the RAW file, reading successive spectra
    pub fn iter(&self) -> RawFileReaderIter<'_> {
        RawFileReaderIter::new(self)
    }

    /// Retrieve the status of the `dotnet` `RawFileReader`
    pub fn status(&self) -> RawFileReaderError {
        self.validate_impl();
        let status_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> u32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("Status"),
            )
            .unwrap();
        let code = status_fn(self.raw_file_reader);
        code.into()
    }
}

#[derive(Debug)]
pub struct RawFileReaderIter<'a> {
    handle: &'a RawFileReader,
    index: usize,
    size: usize,
}

unsafe impl<'a> Send for RawFileReaderIter<'a> {}

impl<'a> RawFileReaderIter<'a> {
    fn new(handle: &'a RawFileReader) -> Self {
        let size = handle.len();
        Self {
            handle,
            index: 0,
            size,
        }
    }
}

impl<'a> Iterator for RawFileReaderIter<'a> {
    type Item = RawSpectrum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let buffer = self.handle.get(self.index);
            self.index += 1;
            buffer
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RawFileReaderIntoIter {
    handle: RawFileReader,
    index: usize,
    size: usize,
}

impl RawFileReaderIntoIter {
    fn new(handle: RawFileReader) -> Self {
        let size = handle.len();
        Self {
            handle,
            index: 0,
            size,
        }
    }
}

impl Iterator for RawFileReaderIntoIter {
    type Item = RawSpectrum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let buffer = self.handle.get(self.index);
            self.index += 1;
            buffer
        } else {
            None
        }
    }
}

impl IntoIterator for RawFileReader {
    type Item = RawSpectrum;

    type IntoIter = RawFileReaderIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        RawFileReaderIntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a RawFileReader {
    type Item = RawSpectrum;

    type IntoIter = RawFileReaderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use std::io;

    use super::*;

    #[test]
    fn test_read() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;

        assert_eq!(handle.len(), 48);
        let buf = handle.get(5).unwrap();
        let descr = buf.view();
        assert_eq!(descr.index(), 5);

        Ok(())
    }

    #[test]
    fn test_read_512kb() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;

        assert_eq!(handle.len(), 48);
        let buf = handle.get(1).unwrap();
        let descr = buf.view();
        assert_eq!(descr.index(), 1);

        Ok(())
    }

    #[test]
    fn test_iter() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;

        let (m1, mn) =
            handle
                .iter()
                .map(|s| s.view().ms_level())
                .fold((0, 0), |(m1, mn), level| {
                    if level > 1 {
                        (m1, mn + 1)
                    } else {
                        (m1 + 1, mn)
                    }
                });

        assert_eq!(m1 + mn, 48);
        assert_eq!(m1, 14);
        assert_eq!(mn, 34);
        Ok(())
    }

    #[test]
    fn test_instrument_model() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;
        let model = handle.instrument_model();

        assert_eq!(model.model(), Some("LTQ FT"));

        Ok(())
    }

    #[test]
    fn test_file_description() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;
        let fd = handle.file_description();
        assert_eq!(fd.sample_id(), Some("1"));
        assert_eq!(fd.source_file(), Some("../tests/data/small.RAW"));
        let counts = fd.spectra_per_ms_level().unwrap();
        assert_eq!(counts.get(0), 14);
        assert_eq!(counts.get(1), 34);
        Ok(())
    }

    #[test]
    fn test_fail_gracefully_opening_non_raw_file() {
        assert!(RawFileReader::open("../test/data/small.mgf").is_err())
    }
}
