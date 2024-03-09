// #![doc = include_str!("../../schema/schema.fbs")]
#![allow(dead_code, unused)]
pub(crate) mod schema_generated;
pub use schema_generated::librawfilereader::{
    AcquisitionT, ActivationT, DissociationMethod, InstrumentConfigurationT, InstrumentModelT,
    Polarity, SpectrumData, SpectrumDescription, SpectrumMode,
};
