#![doc(
    html_favicon_url = "https://observation.tools/img/favicon.svg",
    html_logo_url = "https://observation.tools/img/logo.svg",
    issue_tracker_base_url = "https://github.com/Dig-Doug/observation-tools-client/issues/"
)]
//!
//! # Quickstart
//!
//! Observation Tools helps you quickly inspect complex data without needing to
//! build your own visualization tools.
//!
//! Integrating Observation Tools into your program takes about 5 minutes. You
//! need to:
//!
//! 1. Create a project
//! 2. Install a client library
//! 3. Export your data
//! 5. Visualize your data
//!
//! ### Organizing your data
//!
//! We use four different concepts to organize your data:
//!
//! - **Artifacts** are individual pieces of data, e.g. an image
//! - **Artifact groups** help organize artifacts and can define how artifacts
//!   can be visualized together.
//! - **Runs** are the top level artifact groups. The normally correspond to one
//!   program execution or http request.
//! - **Projects** allow you to define common settings used across runs.
//!
//! ## Create a project
//!
//! All data uploaded to Observation Tools is associated with a project.
//! Projects have a unique ID that you'll use to initialize the client in the
//! next step.
//!
//! To create a project, do the following:
//! 1. Sign in to the [dashboard](https://app.observation.tools/)
//! 1. Click "Create project"
//!
//! You should see your project's ID on the following screen. You can also find
//! it on the "Settings" page. Project IDs are not sensitive, so you can embed
//! them in your source code.
//!
//! ## Install a client library
//!
//! We have client libraries for a few different languages:
//!
//! | Language | Package | Version|
//! |----------|---------| -------|
//! | Rust     | `observation-tools` | [![Crates.io](https://img.shields.io/crates/v/observation-tools)](https://crates.io/crates/observation-tools) |
//! | JavaScript **experimental** | `@observation-tools/client` | [![npm(scoped)](https://img.shields.io/npm/v/%40observation-tools/client)](https://www.npmjs.com/package/@observation-tools/client) |
//!
//! Don't see the language you're looking for? Let us know! File a [feature request](https://github.com/Dig-Doug/observation-tools-client/issues) or [email us](mailto:help@observation.tools).
//!
//! Install the Rust client with the following command:
//!
//! ```sh
//! cargo add observation-tools
//! ```
//!
//! ## Export your data
//!
//! To start exporting data from your program, we need to set up a client for
//! your project and create a run. After that, we can create groups to organize
//! artifacts during different parts of our program and export artifacts:
//!
//! ```rust
//! use observation_tools::artifacts::Point2Builder;
//! use observation_tools::artifacts::Segment2Builder;
//! use observation_tools::Client;
//! use observation_tools::ClientOptions;
//! use observation_tools::TokenGenerator;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new(
//!         std::env::var("OBSERVATION_TOOLS_PROJECT_ID")?,
//!         ClientOptions::default(),
//!     )?;
//!
//!     /// The name of the run will show up in the UI. You can optionally add key-value metadata to
//!     /// all objects, see [`builders::UserMetadataBuilder::add_metadata`].
//!     let run_uploader = client.create_run("getting-started-example")?;
//!     /// ArtifactGroups are represented as "uploaders"
//!     let uploader_2d = run_uploader.child_uploader_2d("shapes")?;
//!     uploader_2d.create_object2(
//!         "segment2",
//!         Segment2Builder::new(Point2Builder::new(-1.0, 1.0), Point2Builder::new(1.0, -1.0)),
//!     )?;
//!
//!     println!("See the output at: {}", run_uploader.viewer_url());
//!     Ok(())
//! }
//! ```
//!
//! For more information on the types of data you can upload, see the
//! documentation for the [`artifacts`] module.
//!
//! ## Visualize your data
//!
//! To view the exported data, you can either find the run on the [dashboard](https://app.observation.tools/) or generate a direct url with [`groups::RunUploader::viewer_url`].
//!
//! # Examples
//! For more examples, check out the [examples](https://github.com/Dig-Doug/observation-tools-client/tree/main/examples) directory in the repository.
extern crate alloc;
extern crate core;

pub mod artifacts;
mod client;
pub mod groups;
mod run_id;
mod task_handle;
mod throttle_without_access_cookie;
mod token_generator;
mod upload_artifact;
mod util;

pub use crate::client::Client;
pub use crate::client::ClientOptions;
//pub use crate::task_handle::ArtifactUploadHandle;
pub use crate::task_handle::ArtifactUploader2dTaskHandle;
pub use crate::task_handle::ArtifactUploader3dTaskHandle;
pub(crate) use crate::task_handle::BaseArtifactUploaderTaskHandle;
pub use crate::task_handle::GenericArtifactUploaderTaskHandle;
pub use crate::task_handle::PublicArtifactIdTaskHandle;
pub use crate::task_handle::PublicSeriesIdTaskHandle;
pub use crate::task_handle::RunUploaderTaskHandle;
pub use crate::token_generator::TokenGenerator;
pub use crate::util::ClientError;
use observation_tools_common::proto::ArtifactId;
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PublicArtifactId {
    pub(crate) id: ArtifactId,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for
    // getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    let config = if cfg!(debug_assertions) {
        WASMLayerConfigBuilder::new().build()
    } else {
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::WARN)
            .build()
    };
    tracing_wasm::set_as_global_default_with_config(config);

    Ok(())
}
