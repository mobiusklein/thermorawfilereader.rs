use std::ffi::c_void;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, ptr};

use netcorehost::{hostfxr::AssemblyDelegateLoader, pdcstr};

use dotnetrawfilereader_sys::RawVec;

use crate::gen::schema_generated::librawfilereader::root_as_spectrum_description;
use crate::get_runtime;
use crate::schema::{
    root_as_spectrum_description_unchecked, Polarity, PrecursorT, SpectrumDescription, SpectrumMode,
};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawFileReaderError {
    Ok = 0,
    FileNotFound,
    InvalidFormat,

    Error = 999,
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
            _ => Self::Error,
        }
    }
}

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

#[derive(Clone)]
pub struct RawFileReaderHandle {
    raw_file_reader: *mut c_void,
    context: Arc<AssemblyDelegateLoader>,
    size: usize,
}

unsafe impl Send for RawFileReaderHandle {}
unsafe impl Sync for RawFileReaderHandle {}

impl Drop for RawFileReaderHandle {
    fn drop(&mut self) {
        self.close()
    }
}

impl Debug for RawFileReaderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawFileReaderHandle")
            .field("raw_file_reader", &self.raw_file_reader)
            .field("context", &"?")
            .field("size", &self.size)
            .finish()
    }
}

impl RawFileReaderHandle {
    pub fn open<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let context = get_runtime();
        let open_fn = context.get_function_with_unmanaged_callers_only::<fn(text_ptr: *const u8, text_length: i32) -> *mut c_void>(
            pdcstr!("librawfilereader.Exports, librawfilereader"),
            pdcstr!("Open")
        ).unwrap();
        let path: PathBuf = path.into();
        let path = path.to_string_lossy().to_string();
        let raw_file_reader = open_fn(path.as_ptr(), path.len() as i32);

        let mut handle = Self {
            raw_file_reader,
            context,
            size: 0,
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
            RawFileReaderError::Error => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "An unknown error occured",
                ))
            }
        }

        handle.size = handle._impl_len();

        Ok(handle)
    }

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

    fn len(&self) -> usize {
        if self.size != 0 {
            self.size
        } else {
            self._impl_len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn close(&mut self) {
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

    pub fn get(&self, index: usize) -> Option<RawSpectrumWrapper> {
        if index >= self.len() {
            return None
        }
        self.validate_impl();
        let buffer_fn = self
            .context
            .get_function_with_unmanaged_callers_only::<fn(*mut c_void, i32) -> RawVec<u8>>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("SpectrumDescriptionFor"),
            )
            .unwrap();
        let buffer = buffer_fn(self.raw_file_reader, (index as i32) + 1);
        Some(RawSpectrumWrapper::new(buffer))
    }

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
            descr.data().map(|dat| {
                let intens_opt = dat.intensity();
                let intens = intens_opt.as_ref().unwrap();
                let val = intens.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
                println!(
                    "Received {} data points, base peak intensity {}",
                    intens.len(),
                    val
                );
                ()
            });
        } else {
            println!("Spectrum at {index}/{} not found", self.len())
        }
    }

    pub fn iter(&self) -> RawFileReaderIter<'_> {
        RawFileReaderIter::new(self)
    }

    pub fn status(&self) -> RawFileReaderError {
        self.validate_impl();
        let status_fn = self
            .context
            .get_function::<fn(*mut c_void) -> u32>(
                pdcstr!("librawfilereader.Exports, librawfilereader"),
                pdcstr!("Status"),
                pdcstr!("librawfilereader.Exports+StatusFn, librawfilereader"),
            )
            .unwrap();
        let code = status_fn(self.raw_file_reader);
        code.into()
    }
}

#[derive(Debug)]
pub struct RawFileReaderIter<'a> {
    handle: &'a RawFileReaderHandle,
    index: usize,
    size: usize,
}

unsafe impl<'a> Send for RawFileReaderIter<'a> {}

impl<'a> RawFileReaderIter<'a> {
    fn new(handle: &'a RawFileReaderHandle) -> Self {
        let size = handle.len();
        Self {
            handle,
            index: 0,
            size,
        }
    }
}

impl<'a> Iterator for RawFileReaderIter<'a> {
    type Item = RawSpectrumWrapper;

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
    handle: RawFileReaderHandle,
    index: usize,
    size: usize,
}

impl RawFileReaderIntoIter {
    fn new(handle: RawFileReaderHandle) -> Self {
        let size = handle.len() as usize;
        Self {
            handle,
            index: 0,
            size,
        }
    }
}

impl Iterator for RawFileReaderIntoIter {
    type Item = RawSpectrumWrapper;

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

impl IntoIterator for RawFileReaderHandle {
    type Item = RawSpectrumWrapper;

    type IntoIter = RawFileReaderIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        RawFileReaderIntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a RawFileReaderHandle {
    type Item = RawSpectrumWrapper;

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
        let handle = RawFileReaderHandle::open("../tests/data/small.RAW")?;

        assert_eq!(handle.len(), 48);
        let buf = handle.get(5).unwrap();
        let descr = buf.view();
        assert_eq!(descr.index(), 5);

        Ok(())
    }

    #[test]
    fn test_read_512kb() -> io::Result<()> {
        let handle = RawFileReaderHandle::open("../tests/data/small.RAW")?;

        assert_eq!(handle.len(), 48);
        let buf = handle.get(1).unwrap();
        let descr = buf.view();
        assert_eq!(descr.index(), 1);

        Ok(())
    }

    #[test]
    fn test_iter() -> io::Result<()> {
        let handle = RawFileReaderHandle::open("../tests/data/small.RAW")?;

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
}
