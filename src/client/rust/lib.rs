extern crate alloc;
extern crate core;

pub mod builders;
mod client;
mod generated;
mod run_id;
mod task_handle;
mod task_loop;
mod token_generator;
pub mod uploaders;
mod util;

pub use crate::client::Client;
pub use crate::client::ClientOptions;
use crate::generated::ArtifactId;
pub use crate::task_handle::ArtifactUploader2dTaskHandle;
pub use crate::task_handle::ArtifactUploader3dTaskHandle;
pub(crate) use crate::task_handle::BaseArtifactUploaderTaskHandle;
pub use crate::task_handle::GenericArtifactUploaderTaskHandle;
pub use crate::task_handle::PublicArtifactIdTaskHandle;
pub use crate::task_handle::PublicSeriesIdTaskHandle;
pub use crate::task_handle::RunUploaderTaskHandle;
pub use crate::token_generator::TokenGenerator;
pub use crate::util::ClientError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
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
