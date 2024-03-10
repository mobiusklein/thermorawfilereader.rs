use std::{collections::HashMap, io, marker::PhantomData, path::PathBuf};

use mzdata::{
    impl_metadata_trait,
    io::OffsetIndex,
    meta::{
        Component, ComponentType, DataProcessing, FileDescription, InstrumentConfiguration,
        Software, SourceFile,
    },
    params::{ControlledVocabulary, Unit},
    prelude::*,
    spectrum::{
        ActivationMethod, ArrayType, BinaryArrayMap, BinaryDataArrayType, DataArray,
        MultiLayerSpectrum, Precursor, ScanEvent, ScanPolarity, ScanWindow, SelectedIon,
        SignalContinuity,
    },
    Param,
};
use mzpeaks::{peak_set::PeakSetVec, prelude::*, CentroidPeak, DeconvolutedPeak, MZ};

use thermorawfilereader::schema::{
    IonizationMode, MassAnalyzer, Polarity, SpectrumData, SpectrumMode,
};
use thermorawfilereader::{
    schema::{AcquisitionT, DissociationMethod, PrecursorT},
    RawFileReader,
};

macro_rules! param {
    ($name:expr, $acc:expr) => {
        ControlledVocabulary::MS.const_param_ident($name, $acc)
    };
}

pub struct ThermoRaw<
    C: CentroidLike + Default + From<CentroidPeak> = CentroidPeak,
    D: DeconvolutedCentroidLike + Default = DeconvolutedPeak,
> {
    pub path: PathBuf,
    handle: RawFileReader,
    index: usize,
    spectrum_index: OffsetIndex,
    file_description: FileDescription,
    instrument_configurations: HashMap<u32, InstrumentConfiguration>,
    components_to_instrument_id: HashMap<(IonizationMode, MassAnalyzer), u32>,
    softwares: Vec<Software>,
    data_processings: Vec<DataProcessing>,
    _c: PhantomData<C>,
    _d: PhantomData<D>,
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default>
    MZFileReader<C, D, MultiLayerSpectrum<C, D>> for ThermoRaw<C, D>
{
    fn construct_index_from_stream(&mut self) -> u64 {
        self.len() as u64
    }

    #[allow(unused)]
    fn open_file(source: std::fs::File) -> Self {
        panic!("Cannot read a Thermo RAW file from an open file handle, only directly from a path")
    }

    fn open_path<P>(path: P) -> io::Result<Self>
    where
        P: Into<std::path::PathBuf> + Clone,
    {
        Self::new(&path.into())
    }
}

fn make_native_id(index: i32) -> String {
    format!(
        "controllerType=0 controllerNumber=1 scan={}",
        index as usize + 1
    )
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default>
    ThermoRaw<C, D>
{
    fn make_file_description(path: &PathBuf) -> FileDescription {
        let mut sf = SourceFile::default();
        sf.name = path.file_name().unwrap().to_string_lossy().to_string();
        sf.location = format!("file:///{}", path.parent().unwrap().display());
        sf.id = "SF1".to_string();
        sf.file_format = Some(
            ControlledVocabulary::MS
                .const_param_ident("Thermo RAW format", 1000563)
                .into(),
        );
        sf.id_format = Some(
            ControlledVocabulary::MS
                .const_param_ident("Thermo nativeID format", 1000768)
                .into(),
        );

        let file_description = FileDescription::new(vec![], vec![sf]);
        file_description
    }

    fn make_instrument_configuration(
        handle: &RawFileReader,
    ) -> (
        Software,
        HashMap<u32, InstrumentConfiguration>,
        HashMap<(IonizationMode, MassAnalyzer), u32>,
    ) {
        let descr = handle.instrument_model();

        let mut sw = Software::default();
        sw.id = "thermo_xcalibur".to_string();
        sw.version = descr.software_version().unwrap().to_string();
        sw.add_param(
            ControlledVocabulary::MS
                .const_param_ident("Xcalibur", 1000532)
                .into(),
        );

        let mut configs = HashMap::new();
        let mut components_to_instrument_id = HashMap::new();

        for (i, vconf) in descr.configurations().into_iter().flatten().enumerate() {
            let mut config = InstrumentConfiguration::default();

            let mut ion_source = Component::default();
            ion_source.order = 0;
            ion_source.component_type = ComponentType::IonSource;
            match vconf.ionization_mode() {
                IonizationMode::CardNanoSprayIonization | IonizationMode::NanoSpray => {
                    ion_source.add_param(param!("nanospray inlet", 1000485).into());
                }
                IonizationMode::ElectroSpray => {
                    ion_source.add_param(param!("electrospray inlet", 1000057).into());
                }
                IonizationMode::ThermoSpray => {
                    ion_source.add_param(param!("thermospray inlet", 1000069).into());
                }
                IonizationMode::FastAtomBombardment => {
                    ion_source
                        .add_param(param!("continuous flow fast atom bombardment", 1000055).into());
                }
                _ => {}
            }
            ion_source.add_param(
                match vconf.ionization_mode() {
                    IonizationMode::CardNanoSprayIonization | IonizationMode::NanoSpray => {
                        param!("nanoelectrospray", 1000398)
                    }
                    IonizationMode::ElectroSpray => {
                        param!("electrospray ionization", 1000073)
                    }
                    IonizationMode::AtmosphericPressureChemicalIonization => {
                        param!("atmospheric pressure chemical ionization", 1000070)
                    }
                    IonizationMode::FastAtomBombardment => {
                        param!("fast atom bombardment ionization", 1000074)
                    }
                    IonizationMode::GlowDischarge => {
                        param!("glow discharge ionization", 1000259)
                    }
                    IonizationMode::ElectronImpact => {
                        param!("electron ionization", 1000389)
                    }
                    IonizationMode::MatrixAssistedLaserDesorptionIonization => {
                        param!("matrix-assisted laser desorption ionization", 1000075)
                    }
                    IonizationMode::ChemicalIonization => {
                        param!("chemical ionization", 1000071)
                    }
                    _ => {
                        param!("ionization type", 1000008)
                    }
                }
                .into(),
            );
            config.components.push(ion_source);

            let mut analyzer = Component::default();
            analyzer.order = 1;
            analyzer.component_type = ComponentType::Analyzer;

            analyzer.add_param(
                match vconf.mass_analyzer() {
                    MassAnalyzer::ITMS => {
                        param!("radial ejection linear ion trap", 1000083)
                    }
                    MassAnalyzer::FTMS => {
                        param!("orbitrap", 1000484)
                    }
                    MassAnalyzer::ASTMS => {
                        param!("asymmetric track lossless time-of-flight analyzer", 1003379)
                    }
                    // MassAnalyzer::TOFMS => {}
                    // MassAnalyzer::TQMS => {}
                    // MassAnalyzer::SQMS => {}
                    // MassAnalyzer::Sector => {}
                    _ => param!("mass analyzer type", 1000443),
                }
                .into(),
            );
            config.components.push(analyzer);

            let mut detector = Component::default();
            detector.order = 2;
            detector.component_type = ComponentType::Detector;
            detector.add_param(
                match vconf.mass_analyzer() {
                    MassAnalyzer::ITMS | MassAnalyzer::ASTMS => {
                        param!("electron multiplier", 1000253)
                    }
                    MassAnalyzer::FTMS => {
                        param!("inductive detector", 1000624)
                    }
                    _ => param!("detector type", 1000026),
                    // MassAnalyzer::TOFMS => {}
                    // MassAnalyzer::TQMS => {}
                    // MassAnalyzer::SQMS => {}
                    // MassAnalyzer::Sector => {}
                }
                .into(),
            );
            config.components.push(detector);

            if let Some(serial) = descr.serial_number() {
                config.add_param(ControlledVocabulary::MS.param_val("1000529", "name", serial));
            }
            config.software_reference = sw.id.clone();
            config.id = i as u32;

            components_to_instrument_id
                .insert((vconf.ionization_mode(), vconf.mass_analyzer()), i as u32);
            // TODO: map model name terms
            config.add_param(Param::new_key_value(
                "instrument model",
                descr.model().unwrap_or_default(),
            ));

            configs.insert(i as u32, config);
        }
        (sw, configs, components_to_instrument_id)
    }

    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let path: PathBuf = path.into();
        let handle = RawFileReader::open(&path)?;
        let mut spectrum_index: OffsetIndex = OffsetIndex::new("spectrum".to_string());
        (0..handle.len()).for_each(|i| {
            spectrum_index.insert(make_native_id(i as i32), i as u64);
        });

        let file_description = Self::make_file_description(&path);
        let (sw, instrument_configurations, components_to_instrument_id) =
            Self::make_instrument_configuration(&handle);

        Ok(Self {
            path: path,
            handle,
            index: 0,
            spectrum_index,
            file_description,
            instrument_configurations,
            components_to_instrument_id,
            softwares: vec![sw],
            data_processings: vec![],
            _c: PhantomData,
            _d: PhantomData,
        })
    }

    fn populate_precursor(&self, vprec: &PrecursorT, precursor: &mut Precursor) {
        let mut ion = SelectedIon::default();
        ion.mz = vprec.mz();
        ion.intensity = vprec.intensity();
        ion.charge = match vprec.charge() {
            0 => None,
            z => Some(z),
        };
        precursor.add_ion(ion);

        let activation = &mut precursor.activation;
        let vact = vprec.activation();
        activation.energy = vact.collision_energy() as f32;
        match vact.dissociation_method() {
            DissociationMethod::CID => {
                *activation.method_mut() = Some(ActivationMethod::CollisionInducedDissociation);
            }
            DissociationMethod::HCD => {
                *activation.method_mut() =
                    Some(ActivationMethod::BeamTypeCollisionInducedDissociation);
            }
            DissociationMethod::ECD => {
                *activation.method_mut() = Some(ActivationMethod::ElectronCaptureDissociation);
            }
            DissociationMethod::ETD => {
                *activation.method_mut() = Some(ActivationMethod::ElectronTransferDissociation);
            }
            DissociationMethod::ETHCD => {
                *activation.method_mut() = Some(ActivationMethod::ElectronTransferDissociation);
                activation.add_param(
                    ActivationMethod::SupplementalBeamTypeCollisionInducedDissociation.into(),
                );
            }
            DissociationMethod::ETCID => {
                *activation.method_mut() = Some(ActivationMethod::ElectronTransferDissociation);
                activation
                    .add_param(ActivationMethod::SupplementalCollisionInducedDissociation.into());
            }
            DissociationMethod::NETD => {
                *activation.method_mut() =
                    Some(ActivationMethod::NegativeElectronTransferDissociation);
            }
            DissociationMethod::MPD => {
                todo!("Need to define MPD")
            }
            DissociationMethod::PTD => {
                todo!("Need to define PTD")
            }
            DissociationMethod::ECCID => {
                *activation.method_mut() = Some(ActivationMethod::ElectronCaptureDissociation);
                activation
                    .add_param(ActivationMethod::SupplementalCollisionInducedDissociation.into());
            }
            DissociationMethod::ECHCD => {
                *activation.method_mut() = Some(ActivationMethod::ElectronCaptureDissociation);
                activation.add_param(
                    ActivationMethod::SupplementalBeamTypeCollisionInducedDissociation.into(),
                )
            }
            _ => {
                *activation.method_mut() = Some(ActivationMethod::CollisionInducedDissociation);
            }
        }

        let iso_window = &mut precursor.isolation_window;
        let vwin = vprec.isolation_window();
        iso_window.lower_bound = vwin.lower() as f32;
        iso_window.target = vwin.target() as f32;
        iso_window.upper_bound = vwin.upper() as f32;

        precursor.precursor_id = Some(make_native_id(vprec.parent_index()));
    }

    fn populate_scan_event(&self, vevent: &AcquisitionT, event: &mut ScanEvent) {
        event.injection_time = vevent.injection_time();
        let window = ScanWindow::new(vevent.low_mz() as f32, vevent.high_mz() as f32);
        event.scan_windows.push(window);
        if let Some(cv) = vevent.compensation_voltage() {
            let mut param =
                ControlledVocabulary::MS.param_val("1001581", "FAIMS compensation voltage", cv);
            param.unit = Unit::Volt;
            event.add_param(param);
        }
        event.add_param(ControlledVocabulary::MS.param_val(
            "1000616",
            "preset scan configuration",
            vevent.scan_event(),
        ));
        let ic_key = (vevent.ionization_mode(), vevent.mass_analyzer());
        if let Some(conf_id) = self.components_to_instrument_id.get(&ic_key) {
            event.instrument_configuration_id = *conf_id;
        }
    }

    fn populate_raw_signal(&self, data: &SpectrumData) -> BinaryArrayMap {
        let mut arrays = BinaryArrayMap::default();

        if let Some(mz) = data.mz() {
            let buffer = mz.bytes();
            let mz_array = DataArray::wrap(
                &ArrayType::MZArray,
                BinaryDataArrayType::Float64,
                buffer.to_vec(),
            );
            arrays.add(mz_array)
        }

        if let Some(intensity) = data.intensity() {
            let buffer = intensity.bytes();
            let intensity_array = DataArray::wrap(
                &ArrayType::IntensityArray,
                BinaryDataArrayType::Float32,
                buffer.to_vec(),
            );
            arrays.add(intensity_array);
        }
        arrays
    }

    fn populate_peaks(&self, data: &SpectrumData) -> PeakSetVec<C, MZ> {
        let mut peaks = PeakSetVec::empty();
        if let (Some(mz), Some(intensity)) = (data.mz(), data.intensity()) {
            for (mz_i, intensity_i) in mz.iter().zip(intensity) {
                let peak = C::from(CentroidPeak::new(mz_i, intensity_i, 0));
                peaks.push(peak);
            }
        }
        peaks
    }

    fn get_spectrum(&mut self, index: usize) -> Option<MultiLayerSpectrum<C, D>> {
        let raw = self.handle.get(index)?;
        let view = raw.view();

        let mut spec = MultiLayerSpectrum::<C, D>::default();

        spec.description.index = view.index() as usize;
        spec.description.id = make_native_id(view.index());
        spec.description.polarity = match view.polarity() {
            Polarity::Negative => ScanPolarity::Negative,
            Polarity::Positive => ScanPolarity::Positive,
            _ => ScanPolarity::Unknown,
        };
        spec.description.ms_level = view.ms_level();
        spec.description.signal_continuity = match view.mode() {
            SpectrumMode::Centroid => SignalContinuity::Centroid,
            SpectrumMode::Profile => SignalContinuity::Profile,
            _ => SignalContinuity::Unknown,
        };

        if let Some(vprec) = view.precursor() {
            let mut prec = Precursor::default();
            self.populate_precursor(vprec, &mut prec);
            spec.description.precursor = Some(prec);
        }

        let event = spec.description.acquisition.first_scan_mut().unwrap();
        event.start_time = view.time();
        if let Some(vacq) = view.acquisition() {
            self.populate_scan_event(&vacq, event);
            if let Some(filter) = view.filter_string() {
                let mut p: Param = param!("filter string", 1000512).into();
                p.value = filter.to_string();
                event.add_param(p);
            }
        }

        if let Some(data) = view.data() {
            if spec.signal_continuity() == SignalContinuity::Centroid {
                spec.peaks = Some(self.populate_peaks(&data));
            } else {
                spec.arrays = Some(self.populate_raw_signal(&data));
            }
        }
        Some(spec)
    }

    pub fn len(&self) -> usize {
        self.handle.len()
    }

    pub fn read_next_spectrum(&mut self) -> Option<MultiLayerSpectrum<C, D>> {
        let i = self.index;
        if i < self.len() {
            let s = self.get_spectrum(i);
            self.index += 1;
            s
        } else {
            None
        }
    }
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default> Iterator
    for ThermoRaw<C, D>
{
    type Item = MultiLayerSpectrum<C, D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next_spectrum()
    }
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default>
    ScanSource<C, D, MultiLayerSpectrum<C, D>> for ThermoRaw<C, D>
{
    fn reset(&mut self) {
        self.index = 0;
    }

    fn get_spectrum_by_id(&mut self, id: &str) -> Option<MultiLayerSpectrum<C, D>> {
        let offset = self.spectrum_index.get(id)?;
        self.get_spectrum(offset as usize)
    }

    fn get_spectrum_by_index(&mut self, index: usize) -> Option<MultiLayerSpectrum<C, D>> {
        self.get_spectrum(index)
    }

    fn get_spectrum_by_time(&mut self, time: f64) -> Option<MultiLayerSpectrum<C, D>> {
        let reload = if self.handle.get_signal_loading() {
            self.handle.set_signal_loading(false);
            true
        } else {
            false
        };
        if let Some(i) = self._offset_of_time(time) {
            if reload {
                self.handle.set_signal_loading(true);
            }
            self.get_spectrum(i as usize)
        } else {
            if reload {
                self.handle.set_signal_loading(true);
            }
            None
        }
    }

    fn get_index(&self) -> &mzdata::io::OffsetIndex {
        &self.spectrum_index
    }

    fn set_index(&mut self, index: mzdata::io::OffsetIndex) {
        self.spectrum_index = index;
    }
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default>
    RandomAccessSpectrumIterator<C, D, MultiLayerSpectrum<C, D>> for ThermoRaw<C, D>
{
    fn start_from_id(&mut self, id: &str) -> Result<&mut Self, SpectrumAccessError> {
        if let Some(i) = self.spectrum_index.get(id) {
            self.index = i as usize;
            Ok(self)
        } else {
            Err(SpectrumAccessError::SpectrumIdNotFound(id.to_string()))
        }
    }

    fn start_from_index(&mut self, index: usize) -> Result<&mut Self, SpectrumAccessError> {
        if index < self.len() {
            self.index = index;
            Ok(self)
        } else {
            Err(SpectrumAccessError::SpectrumIndexNotFound(index))
        }
    }

    fn start_from_time(&mut self, time: f64) -> Result<&mut Self, SpectrumAccessError> {
        let reload = if self.handle.get_signal_loading() {
            self.handle.set_signal_loading(false);
            true
        } else {
            false
        };
        if let Some(i) = self._offset_of_time(time) {
            self.index = i as usize;
            if reload {
                self.handle.set_signal_loading(true);
            }
            Ok(self)
        } else {
            if reload {
                self.handle.set_signal_loading(true);
            }
            Err(SpectrumAccessError::SpectrumNotFound)
        }
    }
}

impl<C: CentroidLike + Default + From<CentroidPeak>, D: DeconvolutedCentroidLike + Default>
    MSDataFileMetadata for ThermoRaw<C, D>
{
    impl_metadata_trait!();

    fn spectrum_count_hint(&self) -> Option<u64> {
        Some(self.spectrum_index.len() as u64)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read() -> io::Result<()> {
        let mut reader =
            ThermoRaw::<CentroidPeak, DeconvolutedPeak>::open_path("../tests/data/small.RAW")?;
        assert_eq!(reader.len(), 48);

        let groups: Vec<_> = reader.groups().collect();
        assert_eq!(groups.len(), 14);

        let spec = reader.get_spectrum_by_index(0).unwrap();
        assert_eq!(spec.peaks().len(), 19913);
        assert!(
            (spec.peaks().tic() - 69381984.0).abs() < 1.0,
            "TIC = {}, diff {}",
            spec.peaks().tic(),
            spec.peaks().tic() - 69381984.0
        );

        let bp = spec.peaks().base_peak();
        assert!(
            (bp.mz - 810.41547).abs() < 1e-3,
            "Base Peak m/z = {}, Diff = {}",
            bp.mz,
            bp.mz - 810.41547
        );

        assert_eq!(spec.ms_level(), 1);
        assert_eq!(spec.signal_continuity(), SignalContinuity::Profile);
        assert_eq!(spec.polarity(), ScanPolarity::Positive);

        let event = spec.acquisition().first_scan().unwrap();
        assert_eq!(0, event.instrument_configuration_id);

        assert!((event.injection_time - 68.227486).abs() < 1e-3);
        assert!((event.start_time - 0.004935).abs() < 1e-3);

        Ok(())
    }
}
