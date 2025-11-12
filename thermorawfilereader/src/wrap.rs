use std::borrow::Cow;
use std::ffi::c_void;
use std::fmt::{Debug, Display};
use std::iter::{FusedIterator, ExactSizeIterator};
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, ptr};

use netcorehost::hostfxr::ManagedFunction;
use netcorehost::{hostfxr::AssemblyDelegateLoader, pdcstr};

use flatbuffers::{root, root_unchecked, Vector};

use dotnetrawfilereader_sys::{try_get_runtime, RawVec};

use crate::constants::{IonizationMode, MSOrder, MassAnalyzer, ScanMode, TraceType};
use crate::schema::{
    root_as_spectrum_description, root_as_spectrum_description_unchecked, AcquisitionT,
    ChromatogramDescription as ChromatogramDescriptionT, ExtendedSpectrumDataT, FileDescriptionT,
    InstrumentMethodT, InstrumentModelT, Polarity, PrecursorT, SpectrumData as SpectrumDataT,
    SpectrumDescription, SpectrumMode, StatusLogCollectionT, TrailerValuesT,
};

macro_rules! view_proxy {
    ($meth:ident) => {
        pub fn $meth(&self) -> Option<&str> {
            self.view().$meth()
        }
    };
    ($meth:ident, $descr:literal) => {
        #[doc=$descr]
        pub fn $meth(&self) -> Option<&str> {
            self.view().$meth()
        }
    };
    ($meth:ident, $descr:literal, $ret:ty) => {
        #[doc=$descr]
        pub fn $meth(&self) -> $ret {
            self.view().$meth()
        }
    };
    ($meth:ident, $ret:ty) => {
        pub fn $meth(&self) -> $ret {
            self.view().$meth()
        }
    };
    ($meth:ident, $descr:literal, $ret:ty, cast) => {
        #[doc=$descr]
        pub fn $meth(&self) -> $ret {
            let data = self.view().$meth();
            #[cfg(target_endian = "big")]
            return Cow::Owned(data.iter().copied().collect());
            #[cfg(target_endian = "little")]
            return Cow::Borrowed(bytemuck::cast_slice(data.bytes()));
        }
    };
    ($meth:ident, $descr:literal, $ret:ty, optcast) => {
        #[doc=$descr]
        pub fn $meth(&self) -> $ret {
            let data = self.view().$meth();
            #[cfg(target_endian = "big")]
            return data.map(|data| Cow::Owned(data.iter().copied().collect()));
            #[cfg(target_endian = "little")]
            return data.map(|data| Cow::Borrowed(bytemuck::cast_slice(data.bytes())));
        }
    };
}

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
    pub fn view(&self) -> SpectrumDescription<'_> {
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

    pub fn ms_order(&self) -> MSOrder {
        MSOrder::from(self.view().ms_order().0)
    }

    pub fn scan_mode(&self) -> ScanMode {
        ScanMode::from(self.view().scan_mode().0)
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
                    return Some(SpectrumData {
                        mz: m,
                        intensity: i,
                    });
                }
            }
        }
        return None;
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
    pub fn acquisition(&self) -> Option<AcquisitionT<'_>> {
        self.view().acquisition()
    }
}


pub struct OwnedSpectrumData {
    data: RawVec<u8>,
}

impl OwnedSpectrumData {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `SpectrumDescription`
    pub fn check(&self) -> bool {
        root::<SpectrumDataT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `SpectrumDescription`
    pub fn raw_view(&self) -> SpectrumDataT<'_> {
        unsafe { root_unchecked::<SpectrumDataT>(&self.data) }
    }

    pub fn view(&self) -> SpectrumData<'_> {
        let view = self.raw_view();
        let mz = view.mz().unwrap_or_default();
        let intensity = view.intensity().unwrap_or_default();
        SpectrumData { mz, intensity }
    }

    pub fn mz(&self) -> Cow<'_, [f64]> {
        self.view().mz()
    }

    pub fn intensity(&self) -> Cow<'_, [f32]> {
        self.view().intensity()
    }

    pub fn is_empty(&self) -> bool {
        self.view().is_empty()
    }

    pub fn iter(&self) -> std::iter::Zip<flatbuffers::VectorIter<'_, f64>, flatbuffers::VectorIter<'_, f32>> {
        self.view().iter()
    }

    pub fn len(&self) -> usize {
        self.view().len()
    }
}


pub struct OwnedSpectrumDataIter {
    inner: OwnedSpectrumData,
    i: usize
}

impl Iterator for OwnedSpectrumDataIter {
    type Item = (f64, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.at(self.i);
        self.i += 1;
        x
    }
}

impl OwnedSpectrumDataIter {
    fn at(&self, i: usize) -> Option<(f64, f32)> {
        let mz = self.inner.mz().get(i).copied()?;
        let int = self.inner.intensity().get(i).copied()?;
        Some((mz, int))
    }
}


/// A sub-set of a [`RawSpectrum`] corresponding to the m/z and intensity arrays
/// of a mass spectrum. All data is borrowed internally from the [`RawSpectrum`]'s
/// buffer.
#[derive(Debug)]
pub struct SpectrumData<'a> {
    mz: Vector<'a, f64>,
    intensity: Vector<'a, f32>,
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

    pub fn iter(
        &self,
    ) -> std::iter::Zip<flatbuffers::VectorIter<'a, f64>, flatbuffers::VectorIter<'a, f32>> {
        let it = self.mz.iter().zip(self.intensity.iter());
        it
    }

    pub fn len(&self) -> usize {
        self.mz.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mz.is_empty()
    }
}

impl<'a> IntoIterator for SpectrumData<'a> {
    type Item = (f64, f32);

    type IntoIter =
        std::iter::Zip<flatbuffers::VectorIter<'a, f64>, flatbuffers::VectorIter<'a, f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct TrailerValues {
    data: RawVec<u8>,
}

impl Debug for TrailerValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items: Vec<_> = self.iter().collect();
        f.debug_struct("TrailerValues")
            .field("data-size", &self.data.len())
            .field("entries", &items)
            .finish()
    }
}

/// A single Trailer Extra value entry.
///
/// This borrows its storage from the original buffer.
#[derive(Debug, Clone, Copy)]
pub struct TrailerValue<'a> {
    /// The human-readable label for this trailer
    pub label: &'a str,
    /// The raw value string for this trailer
    pub value: &'a str,
}

impl TrailerValues {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `TrailerValuesT`
    pub fn check(&self) -> bool {
        root::<TrailerValuesT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `TrailerValuesT`
    pub fn view(&self) -> TrailerValuesT<'_>{
        unsafe { root_unchecked::<TrailerValuesT>(&self.data) }
    }

    pub fn len(&self) -> usize {
        self.view()
            .trailers()
            .into_iter()
            .next()
            .map(|v| v.len())
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.view().trailers().is_some_and(|v| v.is_empty())
    }

    pub fn iter(&self) -> impl Iterator<Item = TrailerValue<'_>> + '_ {
        self.view()
            .trailers()
            .into_iter()
            .flatten()
            .map(|i| -> TrailerValue<'_> {
                TrailerValue {
                    label: i.label().unwrap(),
                    value: i.value().unwrap(),
                }
            })
    }

    pub fn get(&self, index: usize) -> Option<TrailerValue<'_>> {
        self.view().trailers().into_iter().next().and_then(|vec| {
            if index >= vec.len() {
                None
            } else {
                let i = vec.get(index);
                Some(TrailerValue {
                    label: i.label().unwrap(),
                    value: i.value().unwrap(),
                })
            }
        })
    }

    pub fn get_label(&self, label: &str) -> Option<TrailerValue<'_>> {
        self.iter().find(|i| i.label == label)
    }
}

pub struct ExtendedSpectrumData {
    data: RawVec<u8>,
}

impl ExtendedSpectrumData {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `ExtendedSpectrumData`
    pub fn check(&self) -> bool {
        root::<ExtendedSpectrumDataT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `ExtendedSpectrumData`
    pub fn view(&self) -> ExtendedSpectrumDataT<'_> {
        unsafe { root_unchecked::<ExtendedSpectrumDataT>(&self.data) }
    }

    view_proxy!(
        noise,
        "Access the peak-local noise array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        baseline,
        "Access the baseline signal array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        charge,
        "Access the peak charge array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        resolution,
        "Access the peak resolution array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        sampled_noise,
        "Access the sampled noise array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        sampled_noise_baseline,
        "Access the sampled noise baseline array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );

    view_proxy!(
        sampled_noise_mz,
        "Access the sampled noise m/z array, if available",
        Option<Cow<'_, [f32]>>,
        optcast
    );
}

/// The signal trace of a chromatogram
pub struct ChromatogramData<'a> {
    time: Vector<'a, f64>,
    intensity: Vector<'a, f32>,
}

impl<'a> ChromatogramData<'a> {
    /// The time array of the spectrum, in minutes
    pub fn time(&self) -> Cow<'a, [f64]> {
        #[cfg(target_endian = "big")]
        return Cow::Owned(self.time.iter().copied().collect());
        #[cfg(target_endian = "little")]
        Cow::Borrowed(bytemuck::cast_slice(self.time.bytes()))
    }

    /// The intensity array of the spectrum
    pub fn intensity(&self) -> Cow<'a, [f32]> {
        #[cfg(target_endian = "big")]
        return Cow::Owned(self.intensity.iter().copied().collect());
        #[cfg(target_endian = "little")]
        Cow::Borrowed(bytemuck::cast_slice(self.intensity.bytes()))
    }

    /// Iterate over time-intensity pairs
    pub fn iter(
        &self,
    ) -> std::iter::Zip<flatbuffers::VectorIter<'a, f64>, flatbuffers::VectorIter<'a, f32>> {
        let it = self.time.iter().zip(self.intensity.iter());
        it
    }

    pub fn len(&self) -> usize {
        self.time.len()
    }

    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }
}

impl<'a> IntoIterator for ChromatogramData<'a> {
    type Item = (f64, f32);

    type IntoIter =
        std::iter::Zip<flatbuffers::VectorIter<'a, f64>, flatbuffers::VectorIter<'a, f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A wrapper around the `InstrumentModel` FlatBuffer schema. It mirrors the data
/// stored there-in.
///
/// It contains a description of the instrument hardware and control software.
pub struct InstrumentModel {
    data: RawVec<u8>,
}

/// An instrument configuration is a set of hardware components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstrumentConfiguration {
    /// The mass analyzer used in this configuration
    pub mass_analyzer: MassAnalyzer,
    /// The ionization mode used in this configuration
    pub ionization_mode: IonizationMode,
}

impl Display for InstrumentConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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
    pub fn view(&self) -> InstrumentModelT<'_> {
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

    /// The set of distinct instrument configuration names.
    pub fn configurations(&self) -> impl Iterator<Item = InstrumentConfiguration> + '_ {
        self.view().configurations().into_iter().flatten().map(|c| {
            let ionization_mode = c.ionization_mode().0.into();
            let mass_analyzer = c.mass_analyzer().0.into();
            InstrumentConfiguration {
                ionization_mode,
                mass_analyzer,
            }
        })
    }
}

/// A wrapper around the `FileDescription` FlatBuffer schema. It mirrors the data
/// stored there-in.
///
/// It describes the contents of the RAW file and a small amount information about
/// how it was created.
pub struct FileDescription {
    data: RawVec<u8>,
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
    pub fn view(&self) -> FileDescriptionT<'_> {
        root::<FileDescriptionT>(&self.data).unwrap()
    }

    view_proxy!(
        sample_id,
        "The sample identifier provided by the user, if one is present"
    );
    view_proxy!(
        sample_vial,
        "The sample vial name provided by the user or sample handling system, if present"
    );
    view_proxy!(
        sample_comment,
        "The comment describing the sample as provided by the user, if present"
    );
    view_proxy!(
        sample_name,
        "The sample name provided by the user, if one is present"
    );
    view_proxy!(
        source_file,
        "The name of the RAW file being described, as it was recorded by the control software"
    );
    view_proxy!(
        creation_date,
        "The date the RAW file was created, or that the instrument run was performed"
    );

    /// The number of spectra at MS levels 1-10.
    ///
    /// This returns a [`flatbuffers::Vector`] of counts where index `i` corresponds
    /// to the number of MS level `i+1` spectra in the RAW file.
    pub fn spectra_per_ms_level(&self) -> Option<flatbuffers::Vector<'_, u32>> {
        self.view().spectra_per_ms_level()
    }

    /// Trailer headers for the RAW file.
    pub fn trailer_headers(&self) -> Option<Vector<'_, flatbuffers::ForwardsUOffset<&str>>> {
        self.view().trailer_headers()
    }
}

/// The text describing how the instrument was told to operate.
///
/// The text is rendered however the particular instrument reads or
/// presents it, and no consistent formatting can be expected.
///
/// There can be multiple methods per file, such as a chromatography
/// method and a mass spectrometry method. The chromatography method
/// is usually the 0th method and the mass spectrometry method is the
/// 1st method.
pub struct InstrumentMethod {
    data: RawVec<u8>,
}

impl InstrumentMethod {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `InstrumentMethodT`
    pub fn check(&self) -> bool {
        root::<InstrumentMethodT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `InstrumentMethodT`
    pub fn view(&self) -> InstrumentMethodT<'_> {
        root::<InstrumentMethodT>(&self.data).unwrap()
    }

    pub fn index(&self) -> u8 {
        self.view().index()
    }

    pub fn text(&self) -> Option<&str> {
        self.view().text()
    }
}

/// Describes a chromatogram, which is a signal over time.
///
/// The time unit is always in *minutes*, but the signal intensity's
/// unit depends upon the trace type, `TraceTypeT`.
pub struct ChromatogramDescription {
    data: RawVec<u8>,
}

impl ChromatogramDescription {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `ChromatogramDescriptionT`
    pub fn check(&self) -> bool {
        root::<ChromatogramDescriptionT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `ChromatogramDescriptionT`
    pub fn view(&self) -> ChromatogramDescriptionT<'_> {
        root::<ChromatogramDescriptionT>(&self.data).unwrap()
    }

    /// Read the trace type
    pub fn trace_type(&self) -> TraceType {
        self.view().trace_type().into()
    }

    /// The scan number the chromatogram starts at
    pub fn start_index(&self) -> usize {
        (self.view().start_index() as usize).saturating_sub(1)
    }

    /// The scan number the chromatogram ends at
    pub fn end_index(&self) -> usize {
        self.view().end_index() as usize
    }

    /// Access the time-intensity array pair
    pub fn data(&self) -> Option<ChromatogramData<'_>> {
        let view = self.view();
        if let Some(data_view) = view.data() {
            Some(ChromatogramData {
                time: data_view.time().unwrap(),
                intensity: data_view.intensity().unwrap(),
            })
        } else {
            None
        }
    }
}

/// Describes a scan acquisition.
///
/// Acts as a wrapper around [`AcquisitionT`] that translates
/// raw FlatBuffer encodings.
pub struct Acquisition<'a> {
    data: AcquisitionT<'a>,
}

impl<'a> Acquisition<'a> {
    pub fn new(data: AcquisitionT<'a>) -> Self {
        Self { data }
    }

    #[inline(always)]
    pub fn low_mz(&self) -> f64 {
        self.data.low_mz()
    }

    #[inline(always)]
    pub fn high_mz(&self) -> f64 {
        self.data.high_mz()
    }

    #[inline(always)]
    pub fn injection_time(&self) -> f32 {
        self.data.injection_time()
    }

    #[inline(always)]
    pub fn compensation_voltage(&self) -> Option<f32> {
        self.data.compensation_voltages()?.iter().next()
    }

    #[inline(always)]
    pub fn compensation_voltages(&self) -> Option<Vec<f32>> {
        self.data.compensation_voltages().map(|v| v.iter().collect())
    }

    #[inline(always)]
    pub fn mass_analyzer(&self) -> MassAnalyzer {
        self.data.mass_analyzer().0.into()
    }

    #[inline(always)]
    pub fn scan_event(&self) -> i32 {
        self.data.scan_event()
    }

    #[inline(always)]
    pub fn ionization_mode(&self) -> IonizationMode {
        self.data.ionization_mode().0.into()
    }

    #[inline(always)]
    pub fn resolution(&self) -> Option<f32> {
        self.data.resolution()
    }
}

/// A collection of time series information describing the instrument run
pub struct StatusLogCollection {
    data: RawVec<u8>,
}

impl StatusLogCollection {
    pub fn new(data: RawVec<u8>) -> Self {
        Self { data }
    }

    /// Check that the buffer is a valid `StatusLogCollectionT`
    pub fn check(&self) -> bool {
        root::<StatusLogCollectionT>(&self.data).is_ok()
    }

    /// View the underlying buffer as a `StatusLogCollectionT`
    pub fn view(&self) -> StatusLogCollectionT<'_> {
        root::<StatusLogCollectionT>(&self.data).unwrap()
    }

    pub fn str_logs(&self) -> impl Iterator<Item=StatusLog<'_, flatbuffers::ForwardsUOffset<&'_ str>>> {
        self.view().string_logs().into_iter().flatten().map(|log| {
            let name= log.name().map(|name| name.to_string()).unwrap_or_default();
            let time = log.times().unwrap_or_default();
            let values: Vector<'_, flatbuffers::ForwardsUOffset<&str>> = log.values().unwrap_or_default();
            StatusLog {
                name, times: time, values
            }
        })
    }

    pub fn int_logs(&self) -> impl Iterator<Item=StatusLog<'_, i64>> {
        self.view().int_logs().into_iter().flatten().map(|log| {
            let name= log.name().map(|name| name.to_string()).unwrap_or_default();
            let time = log.times().unwrap_or_default();
            let values = log.values().unwrap_or_default();
            StatusLog {
                name, times: time, values
            }
        })
    }

    pub fn float_logs(&self) -> impl Iterator<Item=StatusLog<'_, f64>> {
        self.view().float_logs().into_iter().flatten().map(|log| {
            let name= log.name().map(|name| name.to_string()).unwrap_or_default();
            let time = log.times().unwrap_or_default();
            let values = log.values().unwrap_or_default();
            StatusLog {
                name, times: time, values
            }
        })
    }

    pub fn bool_logs(&self) -> impl Iterator<Item=StatusLog<'_, bool>> {
        self.view().bool_logs().into_iter().flatten().map(|log| {
            let name= log.name().map(|name| name.to_string()).unwrap_or_default();
            let time = log.times().unwrap_or_default();
            let values = log.values().unwrap_or_default();
            StatusLog {
                name, times: time, values
            }
        })
    }
}

pub struct StatusLog<'a, T> {
    pub name: String,
    times: Vector<'a, f64>,
    values: Vector<'a, T>
}

impl<'a, T> StatusLog<'a, T> {
    pub fn new(name: String, time: Vector<'a, f64>, values: Vector<'a, T>) -> Self {
        Self { name, times: time, values }
    }

    pub fn raw_values(&self) -> &Vector<'_, T> {
        &self.values
    }

    pub fn times(&self) -> Cow<'_, [f64]> {
        let data = &self.times;
        #[cfg(target_endian = "big")]
        return Cow::Owned(data.iter().copied().collect());
        #[cfg(target_endian = "little")]
        return Cow::Borrowed(bytemuck::cast_slice(data.bytes()));
    }
}

impl<'a> StatusLog<'a, flatbuffers::ForwardsUOffset<&str>> {
    pub fn strings(&self) -> &Vector<'_, flatbuffers::ForwardsUOffset<&str>> {
        &self.values
    }

    pub fn iter_strings(&self) -> impl Iterator<Item=(f64, &str)> {
        self.times.iter().zip(
            self.strings().iter()
        )
    }
}

impl<'a> StatusLog<'a, bool> {
    pub fn flags(&self) -> &Vector<'_, bool> {
        &self.values
    }

    pub fn iter_flags(&self) -> impl Iterator<Item=(f64, bool)> + '_ {
        self.times.iter().zip(
            self.flags().iter()
        )
    }
}

impl<'a, T: bytemuck::Pod> StatusLog<'a, T> {
    pub fn values(&self) -> Cow<'a, [T]> {
        let data = &self.values;
        #[cfg(target_endian = "big")]
        return Cow::Owned(data.iter().copied().collect());
        #[cfg(target_endian = "little")]
        return Cow::Borrowed(bytemuck::cast_slice(data.bytes()));
    }

    pub fn iter(&self) -> impl Iterator<Item=(f64, T)> + '_ where for<'t> T: flatbuffers::Follow<'t, Inner=T> {
        self.times.iter().zip(self.values.iter())
    }
}

/// A wrapper around a .NET `RawFileReader` instance. It carries a reference to a
/// .NET runtime and a FFI pointer to access data through. The dotnet runtime is
/// controlled via locks and is expected to be thread-safe.
///
/// This object's lifetime controls a shared resource in the .NET runtime.
pub struct RawFileReader {
    /// The token controlling the `RawFileReader` this object references
    raw_file_reader: *mut c_void,
    /// A reference to the .NET runtime
    context: Arc<AssemblyDelegateLoader>,
    /// A cache for the number of spectra in the RAW file
    size: usize,
    include_signal: bool,
    centroid_spectra: bool,
    /// A FFI function pointer to get spectra through.
    vget: ManagedFunction<extern "system" fn(*mut c_void, i32, i32, i32) -> RawVec<u8>>,
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
            .field("include_signal", &self.include_signal)
            .field("centroid_spectra", &self.centroid_spectra)
            .finish()
    }
}

impl RawFileReader {
    /// Open a ThermoFisher RAW file from a path. This may also create the .NET runtime
    /// if this is the first time it was called.
    pub fn open<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let context = try_get_runtime().map_err(|e| match e {
            dotnetrawfilereader_sys::DotNetRuntimeCreationError::FailedToWriteDLLBundle(r) => {
                io::Error::new(io::ErrorKind::Other, r)
            }
            dotnetrawfilereader_sys::DotNetRuntimeCreationError::LoadHostfxrError(r) => {
                io::Error::new(io::ErrorKind::Other, r)
            }
            dotnetrawfilereader_sys::DotNetRuntimeCreationError::HostingError(r) => {
                io::Error::new(io::ErrorKind::Other, r)
            }
            dotnetrawfilereader_sys::DotNetRuntimeCreationError::IOError(r) => r,
        })?;
        let open_fn = context.get_function_with_unmanaged_callers_only::<fn(text_ptr: *const u8, text_length: i32) -> *mut c_void>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Open")
        ).unwrap();
        let path: PathBuf = path.into();
        let path = path.to_string_lossy().to_string();
        let raw_file_reader = open_fn(path.as_ptr(), path.len() as i32);

        let buffer_fn = context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32, i32, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SpectrumDescriptionForWithOptions"),
            )
            .unwrap();

        let mut handle = Self {
            raw_file_reader,
            context,
            include_signal: true,
            centroid_spectra: false,
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
                    format!(
                        "File does not appear to be a valid RAW file. {}",
                        handle.error_message().unwrap_or_default()
                    ),
                ))
            }
            RawFileReaderError::Error | RawFileReaderError::HandleNotFound => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "An unknown error occured {}",
                        handle.error_message().unwrap_or_default()
                    ),
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
        self.include_signal
    }

    /// Set whether or not to retrieve the spectrum signal data when retrieving spectra
    pub fn set_signal_loading(&mut self, value: bool) {
        self.include_signal = value;
    }

    /// Get whether or not to centroid spectra if they were stored in profile mode
    pub fn get_centroid_spectra(&self) -> bool {
        self.centroid_spectra
    }

    /// Set whether or not to centroid spectra if they were stored in profile mode
    pub fn set_centroid_spectra(&mut self, value: bool) {
        self.centroid_spectra = value;
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

    /// Retrieve descriptive metadata about the file and summary measures
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

    /// Read the `index`-th instrument method.
    ///
    /// If no instrument method is found, the Thermo library returns an
    /// empty string. Instead, this returns `None`.
    pub fn instrument_method(&self, index: u8) -> Option<InstrumentMethod> {
        self.validate_impl();
        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("InstrumentMethod"),
            )
            .unwrap();
        let buf = descr_fn(self.raw_file_reader, index as i32);
        root::<InstrumentMethodT>(&buf).unwrap();
        let method = InstrumentMethod::new(buf);
        if method.text().is_none() || method.text().is_some_and(|s| s.is_empty()) {
            None
        } else {
            Some(method)
        }
    }

    /// Get the number of instrument methods that are present in the file
    pub fn instrument_method_count(&self) -> usize {
        self.validate_impl();
        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> u32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("InstrumentMethodCount"),
            )
            .unwrap();
        let n = descr_fn(self.raw_file_reader);
        n as usize
    }

    /// Read the total ion current chromatogram spanning the entire MS run
    pub fn tic(&self) -> ChromatogramDescription {
        self.validate_impl();
        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetTIC"),
            )
            .unwrap();
        let buf = descr_fn(self.raw_file_reader);
        ChromatogramDescription::new(buf)
    }

    /// Read the base peak current chromatogram spanning the entire MS run
    pub fn bpc(&self) -> ChromatogramDescription {
        self.validate_impl();
        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetBPC"),
            )
            .unwrap();
        let buf = descr_fn(self.raw_file_reader);
        ChromatogramDescription::new(buf)
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

    /// Close the RAW file, releasing resources held by the .NET
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
        let buffer = buffer_fn(
            self.raw_file_reader,
            (index as i32) + 1,
            self.include_signal as i32,
            self.centroid_spectra as i32,
        );
        Some(RawSpectrum::new(buffer))
    }

    /// Retrieve extra signal information like the baseline, charge and noise
    /// supplemental arrays for a spectrum.
    ///
    /// ## Note
    /// This method is experimental and may not work reliably.
    pub fn get_extended_spectrum_data(&self, index: usize, include_sampled_noise: bool) -> Option<ExtendedSpectrumData> {
        if index >= self.len() {
            return None;
        }
        self.validate_impl();

        let buffer_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("AdvancedPacketDataFor"),
            )
            .unwrap();

        let buff = buffer_fn(self.raw_file_reader, (index as i32) + 1, include_sampled_noise as i32);
        Some(ExtendedSpectrumData::new(buff))
    }

    /// Read spectrum signal data separately and explicitly without reading all related metadata
    pub fn get_spectrum_data(&self, index: usize, centroid_spectra: bool) -> Option<OwnedSpectrumData> {
        if index >= self.len() {
            return None;
        }
        self.validate_impl();
        let buffer_fn = self.context.get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32, i32) -> RawVec<u8>>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SpectrumDataFor")
        ).unwrap();

        let buff = buffer_fn(self.raw_file_reader, (index as i32) + 1, centroid_spectra as i32);
        Some(OwnedSpectrumData::new(buff))
    }

    /// Get the trailer extra values for scan at `index`.
    ///
    /// The trailer extra values are key-value pairs whose exact meaning
    /// is under dependent upon the instrument and method used. All values
    /// are passed as strings which must be parsed into the appropriate Rust
    /// type to be useful.
    pub fn get_raw_trailers_for(&self, index: usize) -> Option<TrailerValues> {
        if index >= self.len() {
            return None;
        }
        self.validate_impl();

        let buffer_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetRawTrailerValuesFor"),
            )
            .unwrap();

        let buff = buffer_fn(self.raw_file_reader, (index as i32) + 1);
        Some(TrailerValues::new(buff))
    }

    pub fn get_status_logs(&self) -> Option<StatusLogCollection> {
        self.validate_impl();

        let descr_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetStatusLogs"),
            )
            .unwrap();

        let buff = descr_fn(self.raw_file_reader);
        Some(StatusLogCollection::new(buff))
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

    /// Retrieve the status of the .NET `RawFileReader`
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

    /// Retrieve the "file error" status message. This message may
    /// or may not be meaningful depending upon what went wrong.
    pub fn error_message(&self) -> Option<String> {
        self.validate_impl();
        let status_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("GetErrorMessageFor"),
            )
            .unwrap();
        let result = status_fn(self.raw_file_reader);
        if result.len() == 0 || result.len() == 1 && result[0] == 0 {
            return None;
        }
        let message =
            String::from_utf8(result.to_vec()).expect("Failed to decode message, invalid UTF8");
        Some(message)
    }
}

/// Iterator for [`RawFileReader`]
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


/// IntoIterator for [`RawFileReader`]
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

impl<'a> FusedIterator for RawFileReaderIter<'a> {}

impl<'a> ExactSizeIterator for RawFileReaderIter<'a> {
    fn len(&self) -> usize {
        self.size
    }
}

impl FusedIterator for RawFileReaderIntoIter {}

impl ExactSizeIterator for RawFileReaderIntoIter {
    fn len(&self) -> usize {
        self.size
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

        assert_eq!(buf.mode(), SpectrumMode::Centroid);
        let dat = handle.get_spectrum_data(5, false).unwrap();
        assert_eq!(dat.len(), 0);
        let dat = handle.get_spectrum_data(5, true).unwrap();
        assert_eq!(dat.len(), buf.data().unwrap().len());

        Ok(())
    }

    #[test]
    fn test_read_trailers() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;

        assert_eq!(handle.len(), 48);

        let trailers = handle.get_raw_trailers_for(5).unwrap();
        assert!(trailers.iter().all(|kv| !kv.label.ends_with(':')));
        assert_eq!(trailers.len(), 26);

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
    fn test_tic() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;
        let tic = handle.tic();
        assert_eq!(tic.trace_type(), TraceType::TIC);
        assert_eq!(tic.start_index(), 0);
        assert_eq!(tic.end_index(), 48);
        let data = tic.data().unwrap();
        assert_eq!(data.time().len(), 48);
        assert_eq!(data.intensity().len(), 48);
        let expected = 196618480.0f32;
        let total = data.intensity().iter().sum::<f32>();
        assert!(
            (expected - total).abs() < 1e-3,
            "sum = {total}, expected {expected} ({})",
            (expected - total).abs()
        );
        Ok(())
    }

    #[test]
    fn test_bpc() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;
        let bpc = handle.bpc();
        assert_eq!(bpc.trace_type(), TraceType::BasePeak);
        assert_eq!(bpc.start_index(), 0);
        assert_eq!(bpc.end_index(), 48);
        let data = bpc.data().unwrap();
        assert_eq!(data.time().len(), 48);
        assert_eq!(data.intensity().len(), 48);
        let expected = 16132207.0f32;
        let total = data.intensity().iter().sum::<f32>();
        assert!(
            (expected - total).abs() < 1e-3,
            "sum = {total}, expected {expected} ({})",
            (expected - total).abs()
        );
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
    fn test_open_unicode_filename() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small_µ.RAW")?;
        let fd = handle.file_description();
        assert_eq!(fd.sample_id(), Some("1"));
        assert_eq!(fd.source_file(), Some("../tests/data/small_µ.RAW"));
        let counts = fd.spectra_per_ms_level().unwrap();
        assert_eq!(counts.get(0), 14);
        assert_eq!(counts.get(1), 34);
        Ok(())
    }

    #[test]
    fn test_fail_gracefully_opening_non_raw_file() {
        assert!(RawFileReader::open("../test/data/small.mgf").is_err())
    }

    #[test]
    fn test_status_logs() -> io::Result<()> {
        let handle = RawFileReader::open("../tests/data/small.RAW")?;
        let logs = handle.get_status_logs().unwrap();
        assert!(logs.check());
        let view = logs.view();

        let float_stats = view.float_logs().unwrap();
        for stat_log in float_stats.iter() {
            let times = stat_log.times().unwrap();
            let vals = stat_log.values().unwrap();
            // eprintln!("{} {} {:?} {:?}", stat_log.name().unwrap(), times.len(), times.get(0), times.iter().last());
            assert!(times.len() > 0);
            assert_eq!(times.len(), vals.len());
        }

        let str_stats = view.string_logs().unwrap();
        for stat_log in str_stats.iter() {
            let times = stat_log.times().unwrap();
            let vals = stat_log.values().unwrap();
            // eprintln!("{} {} {:?} {:?}", stat_log.name().unwrap(), times.len(), vals.get(0), vals.iter().last());
            assert!(times.len() > 0);
            assert_eq!(times.len(), vals.len());
        }
        Ok(())
    }
}
