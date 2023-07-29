extern crate alloc;

mod api;
mod artifact_uploader_2d;
mod artifact_uploader_3d;
mod base_artifact_uploader;
pub mod builders;
mod client;
mod generic_artifact_uploader;
mod run_id;
mod run_stage_uploader;
mod run_uploader;
mod task_handler;
mod token_generator;
mod upload_artifact_task;
mod util;

pub use crate::artifact_uploader_2d::ArtifactUploader2d;
pub use crate::artifact_uploader_3d::ArtifactUploader3d;
pub use crate::builders::UserMetadataBuilder;
pub use crate::client::Client;
pub use crate::client::ClientOptions;
pub use crate::generic_artifact_uploader::GenericArtifactUploader;
pub use crate::run_stage_uploader::RunStageUploader;
pub use crate::run_uploader::RunUploader;
pub use crate::token_generator::TokenGenerator;
use artifacts_api_rust_proto::ArtifactId;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PublicArtifactId {
    pub(crate) id: ArtifactId,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    // Add this line:
    tracing_wasm::set_as_global_default();

    Ok(())
}
