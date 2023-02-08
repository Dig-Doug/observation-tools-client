use crate::util::{encode_id_proto, new_uuid_proto, GenericError};
use artifacts_api_rust_proto::StaticSourceDataManifestEntry;
use artifacts_api_rust_proto::StaticSourceDataSourceReference;
use artifacts_api_rust_proto::StaticSourceDataVersionEntry;
use artifacts_api_rust_proto::{StaticSourceDataManifest, StaticSourceDataManifestEntryId};
use custom_error::custom_error;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

custom_error! {#[derive(Clone)] pub StaticSourceDataError
    SourceFileNotFound { file_name: String } = "Source file not found: {}",
    UpdateMissingParentEntry { parent_id: String } = "Update missing parent entry: {parent_id}",
    SourceModifiedBeforeUpdateApplied { entry_id: String, file_name: String} = "Source modified before update applied: {entry_id} {file_name}",
}

fn get_static_source_data_path(
    reference: &StaticSourceDataSourceReference,
) -> Result<PathBuf, StaticSourceDataError> {
    let source_path = PathBuf::from(&reference.file_name);
    if !source_path.exists() {
        return Err(StaticSourceDataError::SourceFileNotFound {
            file_name: source_path.to_string_lossy().to_string(),
        });
    }
    Ok(source_path)
}

struct StaticSourceDataManifestUpdate {
    entry_id: String,
    new_version: StaticSourceDataVersionEntry,
    path: PathBuf,
}

fn calculate_manifest_updates(
    manifest: &StaticSourceDataManifest,
) -> Result<Vec<StaticSourceDataManifestUpdate>, GenericError> {
    let mut new_versions = Vec::new();
    for entry in manifest.entries.iter() {
        let source_path = get_static_source_data_path(&entry.source.as_ref().unwrap_or_default())?;
        let source_hash = {
            let mut input = File::open(&source_path)?;
            let mut hasher = Sha256::new();
            io::copy(&mut input, &mut hasher)?;
            base64::encode(hasher.finalize())
        };

        if entry.versions.iter().any(|v| &v.hash == &source_hash) {
            continue;
        }

        new_versions.push(StaticSourceDataManifestUpdate {
            entry_id: entry.id.clone(),
            new_version: {
                let mut new_version = StaticSourceDataVersionEntry::new();
                new_version.hash = source_hash;
                new_version
            },
            path: source_path,
        });
    }
    Ok(new_versions)
}

fn apply_update_to_manifest(
    manifest: &mut StaticSourceDataManifest,
    update: &StaticSourceDataManifestUpdate,
) -> Result<(), GenericError> {
    let entry = manifest
        .entries
        .iter_mut()
        .find(|e| e.id == update.entry_id);
    match entry {
        Some(entry) => Ok(entry.versions.push(update.new_version.clone())),
        None => Err(StaticSourceDataError::UpdateMissingParentEntry {
            parent_id: update.entry_id.clone(),
        }
        .into()),
    }
}
