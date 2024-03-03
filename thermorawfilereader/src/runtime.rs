#![allow(unused)]
use std::io;
use std::fs;
use std::path;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use tempdir::{self, TempDir};
use include_dir::{Dir, include_dir};

use netcorehost::{
    nethost, pdcstring::PdCString,
    hostfxr::AssemblyDelegateLoader,
};

use crate::buffer::configure_allocator;

static DOTNET_LIB_DIR: Dir<'_> = include_dir!("lib/");

const PREFIX: &'static str = "librawfilereader/bin/Release";

fn runtime_cfg(version: &str) -> PdCString {
    format!("{PREFIX}/{version}/librawfilereader.runtimeconfig.json").parse().unwrap()
}

fn assembly(version: &str) -> PdCString {
    format!("{PREFIX}/{version}/librawfilereader.dll").parse().unwrap()
}

const NET_VERSION: &str = "net7.0";


#[derive(Debug)]
pub enum BundleStore {
    TempDir(TempDir),
    Path(PathBuf)
}


#[derive()]
pub struct DotNetLibraryBundle {
    dir: BundleStore,
    assembly_loader: RwLock<Option<Arc<AssemblyDelegateLoader>>>
}

impl Default for DotNetLibraryBundle {
    fn default() -> Self {
        Self::new(None).unwrap()
    }
}

impl DotNetLibraryBundle {
    pub fn new(dir: Option<&str>) -> io::Result<Self> {
        let dir = if let Some(path) = dir {
            let path = PathBuf::from(path);
            if path.exists() {
                BundleStore::Path(path)
            } else {
                BundleStore::TempDir(TempDir::new(path.to_str().unwrap())?)
            }
        } else {
            BundleStore::TempDir(TempDir::new(concat!("rawfilereader_libs_", env!("CARGO_PKG_VERSION")))?)
        };
        Ok(Self { dir, assembly_loader: RwLock::new(None) })
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
        let a = self.assembly_loader.read().map(|a| a.clone().unwrap()).unwrap();
        return a
    }

    pub fn write_bundle(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn create_runtime(&self) -> Arc<AssemblyDelegateLoader> {
        let hostfxr = nethost::load_hostfxr().unwrap();

        let runtime_path = self.path().join("librawfilereader.runtimeconfig.json");
        let runtime_path_encoded: PdCString = runtime_path.to_string_lossy().parse().unwrap();

        let context = hostfxr
            .initialize_for_runtime_config(
                runtime_path_encoded
            )
            .unwrap();

        let assembly_path = self.path().join("librawfilereader.dll");
        let assembly_path_encoded: PdCString = assembly_path.to_string_lossy().parse().unwrap();

        let delegate_loader = Arc::new(context
            .get_delegate_loader_for_assembly(
                assembly_path_encoded
            )
            .unwrap());

        delegate_loader
    }
}


static BUNDLE: OnceLock<DotNetLibraryBundle> = OnceLock::new();


pub fn get_runtime() -> Arc<AssemblyDelegateLoader> {
    let bundle = BUNDLE.get_or_init(|| {
        DotNetLibraryBundle::default()
    });

    bundle.runtime()
}


pub fn load_runtime() -> Arc<AssemblyDelegateLoader> {
    let hostfxr = nethost::load_hostfxr().unwrap();
    let context = hostfxr
        .initialize_for_runtime_config(runtime_cfg(NET_VERSION))
        .unwrap();

    let delegate_loader = Arc::new(context
        .get_delegate_loader_for_assembly(assembly(NET_VERSION))
        .unwrap());

    DOTNET_LIB_DIR.entries().iter().for_each(|e| {
        eprintln!("Entry {}", e.path().display());
    });

    configure_allocator(&delegate_loader);
    delegate_loader
}