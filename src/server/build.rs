use anyhow::Context;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::canonicalize;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

// TODO(doug): Fix this
const PNPM: &str = "/home/doug/.local/share/pnpm/pnpm";

fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=storage/sqlite/migrations");
    println!("cargo:rerun-if-changed=../site");

    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR"),
    );
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Failed to get OUT_DIR"));
    println!("Out dir: {:?}", out_dir);

    build_site_archive(manifest_dir.clone(), out_dir.clone())?;

    Ok(())
}

fn build_site_archive(manifest_dir: PathBuf, out_dir: PathBuf) -> Result<(), anyhow::Error> {
    println!("Building site");
    let input_site_dir = canonicalize(manifest_dir.join("../site"))?;
    let output_site_dir = out_dir.join("site");
    Command::new(PNPM)
        .current_dir(&input_site_dir)
        .env("OUT_DIR", output_site_dir.join("build"))
        .arg("build")
        .output()
        .with_context(|| "Building site")?;

    println!("Copying site files");
    for file in &["index.js", "package.json", "pnpm-lock.yaml"] {
        std::fs::copy(input_site_dir.join(file), output_site_dir.join(file))
            .with_context(|| format!("Copying file {}", file))?;
    }

    println!("Installing site dependencies");
    Command::new(PNPM)
        .current_dir(&output_site_dir)
        .arg("install")
        .arg("--prod")
        .arg("--shamefully-hoist")
        .arg("--force")
        .output()
        .with_context(|| "Installing site dependencies")?;

    println!("Archiving site");
    let tar_gz = File::create(out_dir.join("site.tar.gz"))?;
    let enc = GzEncoder::new(tar_gz, Compression::best());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("site", &output_site_dir)
        .with_context(|| "Appending site dir")?;
    tar.finish()?;
    Ok(())
}
