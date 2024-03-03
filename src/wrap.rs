use std::ffi::c_void;
use std::fmt::Display;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;

use netcorehost::{
    nethost, pdcstr, pdcstring::PdCString,
    hostfxr::AssemblyDelegateLoader,
};

use crate::gen::schema_generated::librawfilereader::root_as_spectrum_description;
use crate::buffer::{RawVec, configure_allocator};
use crate::schema::{root_as_spectrum_description_unchecked, Polarity, PrecursorT, SpectrumDescription, SpectrumMode};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawFileReaderError {
    Ok = 0,
    FileNotFound,
    InvalidFormat,

    Error = 999
}

impl Display for RawFileReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<u32> for RawFileReaderError {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::FileNotFound,
            2 => Self::InvalidFormat,
            _ => Self::Error
        }
    }
}

#[allow(unused)]
pub struct RawSpectrumWrapper {
    data: RawVec<u8>,
}

#[allow(unused)]
impl RawSpectrumWrapper {
    pub fn new(data: RawVec<u8>) -> Self {
        root_as_spectrum_description(&data).unwrap();
        Self { data }
    }

    pub fn view(&self) -> SpectrumDescription {
        unsafe { root_as_spectrum_description_unchecked(&self.data) }
    }

    pub fn index(&self) -> usize {
        self.view().index() as usize
    }

    pub fn ms_level(&self) -> u8 {
        self.view().ms_level()
    }

    pub fn time(&self) -> f64 {
        self.view().time()
    }

    pub fn polarity(&self) -> Polarity {
        self.view().polarity()
    }

    pub fn mode(&self) -> SpectrumMode {
        self.view().mode()
    }

    pub fn precursor(&self) -> Option<&PrecursorT> {
        self.view().precursor()
    }

}

pub struct RawFileReaderHandle {
    raw_file_reader: *mut c_void,
    context: Arc<AssemblyDelegateLoader>
}

impl Drop for RawFileReaderHandle {
    fn drop(&mut self) {
        self.close()
    }
}

impl RawFileReaderHandle {
    pub fn open<P: Into<PathBuf>>(context: Arc<AssemblyDelegateLoader>, path: P) -> Self {
        let open_fn = context.get_function::<fn(text_ptr: *const u8, text_length: i32) -> *mut c_void>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Open"),
            pdcstr!("librawfilereader.Exports+OpenFn, librawfilereader")
        ).unwrap();
        let path: PathBuf = path.into();
        let path = path.to_string_lossy().to_string();
        let raw_file_reader = open_fn(path.as_ptr(), path.len() as i32);

        Self {
            raw_file_reader,
            context
        }
    }

    pub fn first_spectrum(&self) -> i32 {
        self.validate_impl();
        let index_fn = self.context.get_function::<fn(*mut c_void) -> i32>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("FirstSpectrum"),
            pdcstr!("librawfilereader.Exports+SpectrumIndexFn, librawfilereader")
        ).unwrap();
        index_fn(self.raw_file_reader)
    }

    pub fn last_spectrum(&self) -> i32 {
        let index_fn = self.context.get_function::<fn(*mut c_void) -> i32>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("LastSpectrum"),
            pdcstr!("librawfilereader.Exports+SpectrumIndexFn, librawfilereader")
        ).unwrap();
        index_fn(self.raw_file_reader)
    }

    #[inline]
    fn validate_impl(&self) {
        if self.raw_file_reader.is_null() {
            panic!("Internal handle already closed.")
        }
    }

    pub fn len(&self) -> i32 {
        self.validate_impl();
        let index_fn = self.context.get_function::<fn(*mut c_void) -> i32>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SpectrumCount"),
            pdcstr!("librawfilereader.Exports+SpectrumIndexFn, librawfilereader")
        ).unwrap();
        index_fn(self.raw_file_reader)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn close(&mut self) {
        if !self.raw_file_reader.is_null() {
            let close_fn = self.context.get_function::<fn(*mut c_void)>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("Close"),
                pdcstr!("librawfilereader.Exports+CloseFn, librawfilereader")
            ).unwrap();
            close_fn(self.raw_file_reader);
            self.raw_file_reader = ptr::null_mut();
        }
    }

    pub fn get(&self, index: i32) -> RawSpectrumWrapper {
        self.validate_impl();
        let buffer_fn = self.context.get_function::<fn(*mut c_void, i32) -> RawVec<u8>>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("SpectrumDescriptionFor"),
            pdcstr!("librawfilereader.Exports+BufferFn, librawfilereader")
        ).unwrap();
        let buffer = buffer_fn(self.raw_file_reader, index);
        RawSpectrumWrapper::new(buffer)
    }

    pub fn describe(&self, index: i32) {
        let buf = self.get(index);
        let descr = buf.view();
        eprintln!("{}|{:?} -> {:?} | has data? {}", descr.index(), descr.polarity(), descr.precursor(), descr.data().is_some());
        descr.data().map(|dat| {
            let intens_opt = dat.intensity();
            let intens = intens_opt.as_ref().unwrap();
            let val = intens.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            eprintln!("Received {} data points, base peak intensity {}", intens.len(), val);
            ()
        });
    }

    pub fn status(&self) -> RawFileReaderError {
        self.validate_impl();
        let status_fn = self.context.get_function::<fn(*mut c_void) -> u32>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Status"),
            pdcstr!("librawfilereader.Exports+StatusFn, librawfilereader")
        ).unwrap();
        let code = status_fn(self.raw_file_reader);
        code.into()
    }
}


const PREFIX: &'static str = "librawfilereader/bin/Release";

fn runtime_cfg(version: &str) -> PdCString {
    format!("{PREFIX}/{version}/librawfilereader.runtimeconfig.json").parse().unwrap()
}

fn assembly(version: &str) -> PdCString {
    format!("{PREFIX}/{version}/librawfilereader.dll").parse().unwrap()
}

const NET_VERSION: &str = "net7.0";


pub fn load_runtime() -> Arc<AssemblyDelegateLoader> {
    let hostfxr = nethost::load_hostfxr().unwrap();
    let context = hostfxr
        .initialize_for_runtime_config(runtime_cfg(NET_VERSION))
        .unwrap();

    let delegate_loader = Arc::new(context
        .get_delegate_loader_for_assembly(assembly(NET_VERSION))
        .unwrap());

    configure_allocator(&delegate_loader);
    delegate_loader
}