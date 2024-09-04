//! Manage the creation of and access to the self-hosted .NET runtime.
//!
//! The [`DotNetLibraryBundle`] is the main entry point.
//!
//! The environment variable `DOTNET_RAWFILEREADER_BUNDLE_PATH` can be used to set a default location
//! for where DLLs will be written to that persists for recurring use.
use std::fmt::Debug;
use std::fs;
use std::env;
use std::io::{self, prelude::*};
use std::path::{self, Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use include_dir::{include_dir, Dir};
use tempfile::{TempDir, Builder as TempDirBuilder};

use netcorehost::{hostfxr::AssemblyDelegateLoader, nethost, pdcstring::PdCString};

use crate::buffer::configure_allocator;

static DOTNET_LIB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/lib/");

const TMP_NAME: &str = concat!("rawfilereader_libs_", env!("CARGO_PKG_VERSION"));
const DEFAULT_VAR_NAME: &str = "DOTNET_RAWFILEREADER_BUNDLE_PATH";


/// Things that can go wrong while creating a .NET runtime
#[derive(Debug, thiserror::Error)]
pub enum DotNetRuntimeCreationError {
    /// An error might occur while creating the .NET DLL bundle
    #[error("Failed to create a directory on the file system to hold .NET DLLs")]
    FailedToWriteDLLBundle(#[source] io::Error),
    /// An error might occur while loading the Hostfxr layer of the .NET runtime
    #[error("Failed to load hostfxr. Is there a .NET runtime available?")]
    LoadHostfxrError(#[source] #[from] nethost::LoadHostfxrError),
    /// An error might occur while loading the core .NET runtime or library invocation
    #[error("Failed to load .NET host runtime. Is there a .NET runtime available?")]
    HostingError(#[source] #[from] netcorehost::error::HostingError),
    /// Any other I/O error that might occur
    #[error(transparent)]
    IOError(io::Error)
}


/// Represent a directory to store bundled files within.
#[derive(Debug)]
pub enum BundleStore {
    /// Use a temporary directory that will be cleaned up automatically.
    TempDir(TempDir),
    /// Use a specific directory that will persist after the process ends.
    Path(PathBuf),
}

/// A location on the file system and an associated .NET DLL bundle to host a
/// .NET runtime for.
///
/// Uses the `DOTNET_RAWFILEREADER_BUNDLE_PATH` environment variable when a default
/// is required, otherwise creates a temporary directory whose lifespan is linked to this
/// object.
#[derive()]
pub struct DotNetLibraryBundle {
    /// Where to write the DLLs
    dir: BundleStore,
    /// A reference to the actual runtime
    assembly_loader: RwLock<Option<Arc<AssemblyDelegateLoader>>>,
}

impl Debug for DotNetLibraryBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DotNetLibraryBundle").field("dir", &self.dir).field("assembly_loader", &"?").finish()
    }
}

impl Default for DotNetLibraryBundle {
    fn default() -> Self {
        match env::var(DEFAULT_VAR_NAME) {
            Ok(val) => Self::new(Some(&val)).unwrap(),
            Err(err) => {
                match err {
                    env::VarError::NotPresent => Self::new(None).unwrap(),
                    env::VarError::NotUnicode(err) => {
                        eprintln!("Failed to decode `{DEFAULT_VAR_NAME}` {}", err.to_string_lossy());
                        Self::new(None).unwrap()
                    },
                }
            },
        }
    }
}

impl DotNetLibraryBundle {
    /// Create a new bundle directory. If a path string is provided, that path
    /// will be used. Otherwise a temporary directory will be created.
    pub fn new(dir: Option<&str>) -> io::Result<Self> {
        let dir = if let Some(path) = dir {
            let pathbuf = PathBuf::from(path);
            if !pathbuf.exists() {
                fs::create_dir_all(&pathbuf)?;
            }
            BundleStore::Path(pathbuf)
        } else {
            env::var(DEFAULT_VAR_NAME).map(|path| -> io::Result<BundleStore> {
                let pathbuf = PathBuf::from(path);
                if !pathbuf.exists() {
                    fs::create_dir_all(&pathbuf)?;
                }
                Ok(BundleStore::Path(pathbuf))
            }).unwrap_or_else(|_| {
                Ok(BundleStore::TempDir(TempDirBuilder::new().prefix(TMP_NAME).tempdir()?))
            })?
        };
        Ok(Self {
            dir,
            assembly_loader: RwLock::new(None),
        })
    }

    /// Get a path reference to the directory
    pub fn path(&self) -> &path::Path {
        match &self.dir {
            BundleStore::TempDir(d) => d.path(),
            BundleStore::Path(d) => d.as_path(),
        }
    }

    /// Get a reference to the .NET runtime, creating it if one has not yet been created.
    ///
    /// This function will panic if a runtime cannot be found. Calls [`DotNetLibraryBundle::try_runtime`] and unwraps.
    ///
    /// See [`DotNetLibraryBundle::try_create_runtime`] for specific runtime creation
    pub fn runtime(&self) -> Arc<AssemblyDelegateLoader> {
        self.try_create_runtime().unwrap()
    }

    /// Get a reference to the .NET runtime, creating it if one has not yet been created
    /// or return an error if the runtime could not be created.
    ///
    /// See [`DotNetLibraryBundle::try_create_runtime`] for specific runtime creation
    pub fn try_runtime(&self) -> Result<Arc<AssemblyDelegateLoader>, DotNetRuntimeCreationError> {
        if let Ok(mut guard) = self.assembly_loader.write() {
            if guard.is_none() {
                *guard = Some(self.try_create_runtime()?);
            }
        }
        let a = self
            .assembly_loader
            .read()
            .map(|a| a.clone().unwrap())
            .unwrap();
        Ok(a)
    }

    /// Write all of the bundled .NET DLLs to the file system at this location
    pub fn write_bundle(&self) -> io::Result<()> {
        let path = self.path();
        let do_write = if !path.exists() {
            fs::create_dir_all(path)?;
            true
        } else if path.join("checksum").exists(){
            let checksum = fs::read(path.join("checksum"))?;
            let lib_checksum = DOTNET_LIB_DIR.get_file("checksum").unwrap().contents();
            checksum != lib_checksum
        } else {
            true
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

    pub fn try_create_runtime(&self) -> Result<Arc<AssemblyDelegateLoader>, DotNetRuntimeCreationError> {
        let hostfxr = nethost::load_hostfxr()?;
        self.write_bundle().map_err(|e| {
            match e.kind() {
                io::ErrorKind::PermissionDenied | io::ErrorKind::NotFound => DotNetRuntimeCreationError::FailedToWriteDLLBundle(e),
                _ => DotNetRuntimeCreationError::IOError(e),
            }
        })?;

        let runtime_path = self.path().join("librawfilereader.runtimeconfig.json");
        let runtime_path_encoded: PdCString = runtime_path.to_string_lossy().parse().unwrap();

        let context = hostfxr
            .initialize_for_runtime_config(runtime_path_encoded)?;


        let assembly_path = self.path().join("librawfilereader.dll");
        let assembly_path_encoded: PdCString = assembly_path.to_string_lossy().parse().unwrap();

        let delegate_loader = Arc::new(
            context
                .get_delegate_loader_for_assembly(assembly_path_encoded)?
        );

        configure_allocator(&delegate_loader);
        Ok(delegate_loader)
    }
}

static BUNDLE: OnceLock<DotNetLibraryBundle> = OnceLock::new();

/// Set the default runtime directory to `path` that will be accessed by [`get_runtime`]
pub fn set_runtime_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path: &Path = path.as_ref();
    if !path.exists() {
        fs::DirBuilder::new().recursive(true).create(path)?;
    }

    let bundle = DotNetLibraryBundle::new(Some(path.to_str().unwrap())).unwrap();
    BUNDLE.set(bundle).unwrap();
    Ok(())
}

/// Get a reference to a shared .NET runtime and associated DLL bundle
///
/// Panics if a runtime cannot be created. Calls [`try_get_runtime`] and unwraps.
pub fn get_runtime() -> Arc<AssemblyDelegateLoader> {
    try_get_runtime().unwrap()
}


/// Get a reference to a shared .NET runtime and associated DLL bundle
/// or return and error if the runtime cannot be created.
pub fn try_get_runtime() -> Result<Arc<AssemblyDelegateLoader>, DotNetRuntimeCreationError> {
    let bundle = BUNDLE.get_or_init(DotNetLibraryBundle::default);

    bundle.try_runtime()
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bundle_writing() -> io::Result<()> {
        let handle = DotNetLibraryBundle::new(None)?;
        let _runtime = handle.runtime();
        Ok(())
    }
}
