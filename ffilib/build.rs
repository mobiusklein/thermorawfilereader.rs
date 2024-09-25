use cc;
use std::{fs, io::{self, prelude::*}, env, process};

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut fh = fs::File::create("buildlog.txt")?;

    let sdk_path = r"C:\Users\Joshua\.nuget\packages\runtime.win-x64.microsoft.dotnet.ilcompiler\8.0.8\sdk\";
    let include_path = "librawfilereader/bin/Release/net8.0/win-x64/publish/";



    let mut ar = cc::Build::new().get_archiver();

    fh.write_all(format!("Archiver: {ar:?}").as_bytes())?;

    let mut ar_proc = ar.arg(format!("/out:{out_dir}/bootstrapperdll.lib"))
      .arg(format!("{sdk_path}/bootstrapperdll.obj"))
      .stderr(process::Stdio::piped())
      .stdout(process::Stdio::piped())
      .spawn().expect("Failed to spawn linker");

    let mut buf = Vec::new();
    ar_proc.stderr.take().unwrap().read_to_end(&mut buf)?;
    fh.write_all(b"Archiver STDERR:\n")?;
    fh.write_all(&buf)?;
    buf.clear();
    fh.write_all(b"Archiver STDOUT:\n")?;
    ar_proc.stdout.take().unwrap().read_to_end(&mut buf)?;
    fh.write_all(&buf)?;

    println!("cargo:rustc-link-lib=static=librawfilereader");
    println!("cargo:rustc-link-search=native={include_path}");

    println!("cargo:rustc-link-search=native={sdk_path}");
    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rerun-if-changed={}/{}", include_path, "librawfilereader.lib");

    Ok(())
}