#![allow(unused)]
use std::fs;
use std::env;
use std::io::{self, prelude::*};
use std::path::{self, Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use include_dir::{include_dir, Dir};
use tempdir::TempDir;

use netcorehost::{hostfxr::AssemblyDelegateLoader, nethost, pdcstring::PdCString};

use crate::buffer::configure_allocator;

static DOTNET_LIB_DIR: Dir<'_> = include_dir!("dotnetrawfilereader-sys/lib/");

const TMP_NAME: &'static str = concat!("rawfilereader_libs_", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub enum BundleStore {
    TempDir(TempDir),
    Path(PathBuf),
}

#[derive()]
pub struct DotNetLibraryBundle {
    dir: BundleStore,
    assembly_loader: RwLock<Option<Arc<AssemblyDelegateLoader>>>,
}

impl Default for DotNetLibraryBundle {
    fn default() -> Self {
        match env::var("DOTNET_RAWFILEREADER_BUNDLE_PATH") {
            Ok(val) => Self::new(Some(&val)).unwrap(),
            Err(err) => {
                match err {
                    env::VarError::NotPresent => Self::new(None).unwrap(),
                    env::VarError::NotUnicode(err) => {
                        eprintln!("Failed to decode `DOTNET_RAWFILEREADER_BUNDLE_PATH` {}", err.to_string_lossy());
                        Self::new(None).unwrap()
                    },
                }
            },
        }
    }
}

impl DotNetLibraryBundle {
    pub fn new(dir: Option<&str>) -> io::Result<Self> {
        let dir = if let Some(path) = dir {
            let pathbuf = PathBuf::from(path);
            if !pathbuf.exists() {
                fs::create_dir(&pathbuf)?;
            }
            BundleStore::Path(pathbuf)
        } else {
            BundleStore::TempDir(TempDir::new(TMP_NAME)?)
        };
        Ok(Self {
            dir,
            assembly_loader: RwLock::new(None),
        })
    }

    pub fn path(&self) -> &path::Path {
        match &self.dir {
            BundleStore::TempDir(d) => d.path(),
            BundleStore::Path(d) => d.as_path(),
        }
    }

    pub fn runtime(&self) -> Arc<AssemblyDelegateLoader> {
        if let Ok(mut guard) = self.assembly_loader.write() {
            if guard.is_none() {
                *guard = Some(self.create_runtime());
            }
        }
        let a = self
            .assembly_loader
            .read()
            .map(|a| a.clone().unwrap())
            .unwrap();
        return a;
    }

    pub fn write_bundle(&self) -> io::Result<()> {
        let path = self.path();
        let do_write = if !path.exists() {
            fs::create_dir(path)?;
            true
        } else {
            if path.join("checksum").exists(){
                let checksum = fs::read(path.join("checksum"))?;
                let lib_checksum = DOTNET_LIB_DIR.get_file("checksum").unwrap().contents();
                checksum != lib_checksum
            } else {
                true
            }
        };

        if !do_write {
            return Ok(())
        }

        for entry in DOTNET_LIB_DIR.entries() {
            if let Some(data_handle) = entry.as_file() {
                let destintation = path.join(entry.path());
                let mut outhandle = io::BufWriter::new(fs::File::create(destintation)?);
                outhandle.write_all(data_handle.contents())?;
            }
        }

        Ok(())
    }

    pub fn create_runtime(&self) -> Arc<AssemblyDelegateLoader> {
        let hostfxr = nethost::load_hostfxr().unwrap();
        self.write_bundle().unwrap();
        let runtime_path = self.path().join("librawfilereader.runtimeconfig.json");
        let runtime_path_encoded: PdCString = runtime_path.to_string_lossy().parse().unwrap();

        let context = hostfxr
            .initialize_for_runtime_config(runtime_path_encoded)
            .unwrap();

        let assembly_path = self.path().join("librawfilereader.dll");
        let assembly_path_encoded: PdCString = assembly_path.to_string_lossy().parse().unwrap();

        let delegate_loader = Arc::new(
            context
                .get_delegate_loader_for_assembly(assembly_path_encoded)
                .unwrap(),
        );

        configure_allocator(&delegate_loader);

        delegate_loader
    }
}

static BUNDLE: OnceLock<DotNetLibraryBundle> = OnceLock::new();

pub fn set_runtime_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path: &Path = path.as_ref();
    if !path.exists() {
        fs::DirBuilder::new().create(path)?;
    }

    let bundle = DotNetLibraryBundle::new(Some(path.to_str().unwrap())).unwrap();
    BUNDLE.set(bundle);
    Ok(())
}

pub fn get_runtime() -> Arc<AssemblyDelegateLoader> {
    let bundle = BUNDLE.get_or_init(|| DotNetLibraryBundle::default());

    bundle.runtime()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bundle_writing() -> io::Result<()> {
        let handle = DotNetLibraryBundle::new(None)?;
        let runtime = handle.runtime();
        Ok(())
    }
}
