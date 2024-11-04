use glob::glob;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), anyhow::Error> {
    let protos: Vec<PathBuf> = glob("proto/*.proto")?
        .filter_map(|p| p.ok())
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect();
    let root_dir = fs::canonicalize("../../")?;
    prost_build::compile_protos(&protos, &[root_dir])?;
    Ok(())
}
