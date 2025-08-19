use crate::r#gen::schema_generated::librawfilereader::TraceTypeT;

/// This enum mirrors the different types of traces covered in Thermo's RawFileReader library.
///
/// Not all variants are supported by *this* library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum TraceType {
    StartMSChromatogramTraces = -1,
    MassRange = 0,
    TIC = 1,
    BasePeak = 2,
    Fragment = 3,
    Custom = 4,
    PrecursorMass = 5,
    EndMSChromatogramTraces = 6,
    StartAnalogChromatogramTraces = 10,
    Analog1 = 11,
    Analog2 = 12,
    Analog3 = 13,
    Analog4 = 14,
    Analog5 = 15,
    Analog6 = 16,
    Analog7 = 17,
    Analog8 = 18,
    EndAnalogChromatogramTraces = 19,
    StartPDAChromatogramTraces = 20,
    WavelengthRange = 21,
    TotalAbsorbance = 22,
    SpectrumMax = 23,
    EndPDAChromatogramTraces = 24,
    StartUVChromatogramTraces = 30,
    ChannelA = 31,
    ChannelB = 32,
    ChannelC = 33,
    ChannelD = 34,
    ChannelE = 35,
    ChannelF = 36,
    ChannelG = 37,
    ChannelH = 38,
    EndUVChromatogramTraces = 39,
    StartPCA2DChromatogramTraces = 40,
    A2DChannel1 = 41,
    A2DChannel2 = 42,
    A2DChannel3 = 43,
    A2DChannel4 = 44,
    A2DChannel5 = 45,
    A2DChannel6 = 46,
    A2DChannel7 = 47,
    A2DChannel8 = 48,
    EndPCA2DChromatogramTraces = 49,
    EndAllChromatogramTraces = 50
}

impl From<TraceTypeT> for TraceType {
    fn from(value: TraceTypeT) -> Self {
        match value.0 {
            -1 => Self::StartMSChromatogramTraces,
            0 => Self::MassRange,
            1 => Self::TIC,
            2 => Self::BasePeak,
            3 => Self::Fragment,
            4 => Self::Custom,
            5 => Self::PrecursorMass,
            6 => Self::EndMSChromatogramTraces,
            10 => Self::StartAnalogChromatogramTraces,
            11 => Self::Analog1,
            12 => Self::Analog2,
            13 => Self::Analog3,
            14 => Self::Analog4,
            15 => Self::Analog5,
            16 => Self::Analog6,
            17 => Self::Analog7,
            18 => Self::Analog8,
            19 => Self::EndAnalogChromatogramTraces,
            20 => Self::StartPDAChromatogramTraces,
            21 => Self::WavelengthRange,
            22 => Self::TotalAbsorbance,
            23 => Self::SpectrumMax,
            24 => Self::EndPDAChromatogramTraces,
            30 => Self::StartUVChromatogramTraces,
            31 => Self::ChannelA,
            32 => Self::ChannelB,
            33 => Self::ChannelC,
            34 => Self::ChannelD,
            35 => Self::ChannelE,
            36 => Self::ChannelF,
            37 => Self::ChannelG,
            38 => Self::ChannelH,
            39 => Self::EndUVChromatogramTraces,
            40 => Self::StartPCA2DChromatogramTraces,
            41 => Self::A2DChannel1,
            42 => Self::A2DChannel2,
            43 => Self::A2DChannel3,
            44 => Self::A2DChannel4,
            45 => Self::A2DChannel5,
            46 => Self::A2DChannel6,
            47 => Self::A2DChannel7,
            48 => Self::A2DChannel8,
            49 => Self::EndPCA2DChromatogramTraces,
            50 => Self::EndAllChromatogramTraces,
            _ => Self::EndAllChromatogramTraces
        }
    }
}



/// This enum mirrors the different types of ionization modes covered in Thermo's RawFileReader library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum IonizationMode
{
    ElectronImpact = 0,
    ChemicalIonization = 1,
    FastAtomBombardment = 2,
    ElectroSpray = 3,
    AtmosphericPressureChemicalIonization = 4,
    NanoSpray = 5,
    ThermoSpray = 6,
    FieldDesorption = 7,
    MatrixAssistedLaserDesorptionIonization = 8,
    GlowDischarge = 9,
    Any = 10,
    PaperSprayIonization = 11,
    CardNanoSprayIonization = 12,
    IonizationMode1 = 13,
    IonizationMode2 = 14,
    IonizationMode3 = 15,
    IonizationMode4 = 16,
    IonizationMode5 = 17,
    IonizationMode6 = 18,
    IonizationMode7 = 19,
    IonizationMode8 = 20,
    IonizationMode9 = 21,
    IonModeBeyondKnown = 22,
}

impl From<u8> for IonizationMode {
    fn from(value: u8) -> Self {
        match value {
             0 => Self::ElectronImpact,
             1 => Self::ChemicalIonization,
             2 => Self::FastAtomBombardment,
             3 => Self::ElectroSpray,
             4 => Self::AtmosphericPressureChemicalIonization,
             5 => Self::NanoSpray,
             6 => Self::ThermoSpray,
             7 => Self::FieldDesorption,
             8 => Self::MatrixAssistedLaserDesorptionIonization,
             9 => Self::GlowDischarge,
             10 => Self::Any,
             11 => Self::PaperSprayIonization,
             12 => Self::CardNanoSprayIonization,
             13 => Self::IonizationMode1,
             14 => Self::IonizationMode2,
             15 => Self::IonizationMode3,
             16 => Self::IonizationMode4,
             17 => Self::IonizationMode5,
             18 => Self::IonizationMode6,
             19 => Self::IonizationMode7,
             20 => Self::IonizationMode8,
             21 => Self::IonizationMode9,
             22 => Self::IonModeBeyondKnown,
             _ => Self::IonModeBeyondKnown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum MSOrder {
    NeutralGain = -3,
    NeutralLoss = -2,
    ParentScan = -1,
    Any = 0,
    MS = 1,
    MS2 = 2,
    MS3 = 3,
    MS4 = 4,
    MS5 = 5,
    MS6 = 6,
    MS7 = 7,
    MS8 = 8,
    MS9 = 9,
    MS10 = 10,
    Unknown = 999
}


impl From<i16> for MSOrder {
    fn from(value: i16) -> Self {
        match value {
             -3 => Self::NeutralGain,
             -2 => Self::NeutralLoss,
             -1 => Self::ParentScan,
             0 => Self::Any,
             1 => Self::MS,
             2 => Self::MS2,
             3 => Self::MS3,
             4 => Self::MS4,
             5 => Self::MS5,
             6 => Self::MS6,
             7 => Self::MS7,
             8 => Self::MS8,
             9 => Self::MS9,
             10 => Self::MS10,
             999 => Self::Unknown,
            _ => Self::Unknown
        }
    }
}


/// This enum mirrors the different types of mass analyzers in Thermo's RawFileReader library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MassAnalyzer {
    Unknown = 0,
    ITMS = 1,
    TQMS = 2,
    SQMS = 3,
    TOFMS = 4,
    FTMS = 5,
    Sector = 6,
    ASTMS = 7,
}

impl From<u8> for MassAnalyzer {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::ITMS,
            2 => Self::TQMS,
            3 => Self::SQMS,
            4 => Self::TOFMS,
            5 => Self::FTMS,
            6 => Self::Sector,
            7 => Self::ASTMS,
            _ => Self::Unknown
        }
    }
}