mod artifact_uploader_2d;
mod artifact_uploader_3d;
mod base_artifact_uploader;
mod client;
mod generic_artifact_uploader;
mod run_stage_uploader;
mod run_uploader;
mod uploader_stack;
mod user_metadata;
mod api;
mod static_source_data;
mod util;
mod task_handler;
mod upload_artifact_task;

#[cfg(feature = "python")]
use pyo3::prelude::*;
use crate::api::Image2Builder;

pub use crate::artifact_uploader_2d::ArtifactUploader2d;
pub use crate::artifact_uploader_3d::ArtifactUploader3d;
pub use crate::client::Client;
pub use crate::client::ClientOptions;
pub use crate::client::TokenGenerator;
pub use crate::generic_artifact_uploader::GenericArtifactUploader;
pub use crate::run_stage_uploader::RunStageUploader;
pub use crate::run_uploader::RunUploader;
pub use crate::user_metadata::UserMetadataBuilder;

#[cfg(feature = "cpp")]
use crate::client::ffi_new_client;
use crate::uploader_stack::ffi_get_current_group;
use crate::user_metadata::new_user_metadata;

#[cfg(feature = "cpp")]
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Client;
        fn ffi_new_client(project_id: String) -> Box<Client>;
        fn ffi_create_run(&self) -> Box<RunUploader>;
        fn ffi_deserialize_run_stage(&self, serialized: String) -> Box<RunStageUploader>;
    }

    extern "Rust" {
        type RunUploader;
        fn viewer_url(self: &RunUploader) -> &str;
        fn ffi_create_initial_run_stage(
            &self,
            metadata: &UserMetadataBuilder,
        ) -> Box<RunStageUploader>;
    }

    extern "Rust" {
        type UserMetadataBuilder;
        fn new_user_metadata(name: String) -> Box<UserMetadataBuilder>;
        fn add_metadata(&mut self, key: String, value: String) -> &mut UserMetadataBuilder;
    }

    extern "Rust" {
        type RunStageUploader;
        fn ffi_child_uploader(
            &self,
            metadata: &UserMetadataBuilder,
        ) -> Box<GenericArtifactUploader>;
        fn ffi_child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> Box<ArtifactUploader2d>;
        fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String;
    }

    extern "Rust" {
        type GenericArtifactUploader;
        fn ffi_child_uploader(
            &self,
            metadata: &UserMetadataBuilder,
        ) -> Box<GenericArtifactUploader>;
        fn ffi_child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> Box<ArtifactUploader2d>;
        fn ffi_child_uploader_3d(
            &self,
            metadata: &UserMetadataBuilder,
            transform3_bytes: &[u8],
        ) -> Box<ArtifactUploader3d>;
        fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String;
    }

    extern "Rust" {
        type ArtifactUploader2d;
        fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String;
    }

    extern "Rust" {
        type ArtifactUploader3d;
        fn ffi_child_uploader_2d(
            &self,
            metadata: &UserMetadataBuilder,
            to_3d_transform: &[u8],
        ) -> Box<ArtifactUploader2d>;
        fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String;
    }

    extern "Rust" {
        fn ffi_get_current_group() -> Box<GenericArtifactUploader>;
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn observation_tools_client(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<UserMetadataBuilder>()?;
    m.add_class::<Image2Builder>()?;
    Ok(())
}
