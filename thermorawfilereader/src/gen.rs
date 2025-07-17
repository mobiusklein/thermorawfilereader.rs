// #![doc = include_str!("../../schema/schema.fbs")]
#![allow(dead_code, unused, unsafe_op_in_unsafe_fn)]
pub(crate) mod schema_generated;
pub use schema_generated::librawfilereader::{
    AcquisitionT, ActivationT, DissociationMethod, InstrumentConfigurationT, InstrumentModelT,
    Polarity, SpectrumData, SpectrumDescription, SpectrumMode,
};
