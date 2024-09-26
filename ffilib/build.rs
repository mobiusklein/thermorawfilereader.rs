use cc;
use std::{
    env, fs,
    io::{self, prelude::*},
    path::PathBuf,
    process,
};

fn get_dotnet_ilcompiler_sdk() -> io::Result<String> {
    let mut dotproc = process::Command::new("dotnet")
        .args(["nuget", "locals", "global-packages", "-l"])
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to query nuget package cache");

    let mut content = String::new();
    dotproc
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut content)?;
    let sdk_path = if let Some((_, prefix_root)) = content.trim().split_once(":") {
        PathBuf::from(prefix_root.trim())
            .join("runtime.win-x64.microsoft.dotnet.ilcompiler")
            .join("8.0.8")
            .join("sdk")
            .to_string_lossy()
            .to_string()
    } else {
        panic!("Failed to infer dotnet ilcompiler SDK prefix")
    };
    Ok(sdk_path)
}

fn compile_librawfilereader(log_file: &mut fs::File) -> io::Result<String> {
    log_file.write_all(format!("CWD: {}\n\n", env::current_dir()?.display()).as_bytes())?;
    let mut dotproc = process::Command::new("dotnet")
        .args([
            "publish",
            "../librawfilereader/librawfilereader.csproj",
            "-c",
            "Release",
            "--use-current-runtime",
        ])
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("Failed to query nuget package cache");
    let exit_status = dotproc.wait().unwrap();
    writeln!(log_file, "dotnet publish exit code: {exit_status}")?;
    let mut content = String::new();
    dotproc
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut content)?;
    println!("cargo:warning=Exit Status: {exit_status}");
    println!("cargo:warning=STDERR Content: {content}");
    writeln!(log_file, "dotnet publish STDERR: {content}\n\n")?;
    content.clear();
    dotproc
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut content)?;
    println!("cargo:warning=STDOUT Content: {content:?}");
    writeln!(log_file, "dotnet publish STDOUT: {content}\n\n")?;
    let publine = content
        .split('\n')
        .map(|s| s.trim())
        .enumerate()
        .filter(|(_, s)| !s.is_empty())
        .map(|(i, s)| {
            log_file
                .write_all(format!("{i}: {s}\n").as_bytes())
                .unwrap();
            s
        })
        .last()
        .unwrap();
    println!("cargo:warning=Content: {content}");
    println!("cargo:warning=Publine: {publine}");
    let pub_prefix = publine.trim().split(" -> ").last().unwrap().to_string();
    Ok(pub_prefix)
}

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut fh = fs::File::create("buildlog.txt")?;

    let sdk_path = get_dotnet_ilcompiler_sdk()?;
    fh.write_all(format!(".NET SDK: {}\n\n", sdk_path).as_bytes())?;

    let include_path = compile_librawfilereader(&mut fh)?;
    fh.write_all(format!("Library Include Path: {}\n\n", include_path).as_bytes())?;

    let mut ar = cc::Build::new().get_archiver();

    fh.write_all(format!("Archiver: {ar:?}").as_bytes())?;

    let mut ar_proc = ar
        .arg(format!("/out:{out_dir}/bootstrapperdll.lib"))
        .arg(format!("{sdk_path}/bootstrapperdll.obj"))
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn linker");

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
    println!(
        "cargo:rerun-if-changed={}/{}",
        include_path, "librawfilereader.lib"
    );

    Ok(())
}
