//! **Artifact groups** help organize artifacts and can define how artifacts are
//! visualized together. Artifact groups can be generic, meaning they can
//! hold any type of artifact, or they can be specialized to hold only certain
//! types of artifacts, e.g. 2D objects only. Artifacts are added to groups
//! using an **artifact uploader**. Specialized artifact groups have specialized
//! uploaders, e.g. [ArtifactUploader2d] for 2D objects.
//!
//! # Creating artifacts
//! To upload an artifact, you must create the artifact by calling one of the
//! `create` methods on an artifact group. Calling a creation method will return
//! a [crate::ArtifactUploadHandle] and begin uploading the input data
//! asynchronously.
//!
//! ```rust
//! # use observation_tools::{ArtifactUploadHandle, ClientError};
//! # use observation_tools::groups::ArtifactUploader2d;
//! use observation_tools::artifacts::Point2;
//!
//! # async fn create_artifact(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! // Create methods do some data validation and may return an error
//! let upload_handle = uploader.create_object2("my-point", Point2::new(5.0, 3.0))?;
//! // You can optionally wait for the upload to complete
//! upload_handle.wait_for_upload().await?;
//! #   Ok(())
//! # }
//! ```
//!
//! # Updating artifacts
//! A [crate::ArtifactUploadHandle] returned by a creation method will
//! also contain an **artifact updater** that you can use to submit incremental
//! updates to the artifact. Updates are generally coupled with a
//! [crate::artifacts::Series] to keep track of updates over a dimension,
//! e.g. time.
//!
//! ```rust
//! # use observation_tools::{ArtifactUploadHandle, ClientError};
//! # use observation_tools::groups::ArtifactUploader2d;
//! use observation_tools::artifacts::Object2;
//! use observation_tools::artifacts::Point2;
//! use observation_tools::artifacts::Series;
//! use observation_tools::artifacts::SeriesPoint;
//! use observation_tools::artifacts::Transform2;
//!
//! # async fn update_artifact(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! # // TODO(doug): This series example is pretty messy
//! let mut series_builder = Series::new();
//! let algorithm_step_dimension_id = series_builder.add_dimension("algorithm_step");
//! let algorithm_series_id = uploader.series("grid_algorithm", series_builder)?;
//!
//! let handle = uploader.create_object2("my-point", Point2::new(0.0, 0.0))?;
//!
//! for i in 0..10 {
//!     let mut object2: Object2 = Point2::new(i as f64, 0.0).into();
//!     object2.add_transform(Transform2::identity());
//!     object2.set_series_point(&SeriesPoint::new(
//!         &algorithm_series_id,
//!         *algorithm_step_dimension_id,
//!         i as f64,
//!     ));
//! # // TODO(doug): Clean up `&*` syntax
//!     uploader.update_object2(&*handle, object2)?;
//! }
//! #   Ok(())
//! # }
//! ```
mod artifact_uploader_2d;
mod artifact_uploader_3d;
pub(crate) mod base_artifact_uploader;
mod generic_artifact_uploader;
mod run_uploader;

pub use artifact_uploader_2d::ArtifactUploader2d;
pub use artifact_uploader_3d::ArtifactUploader3d;
pub use generic_artifact_uploader::GenericArtifactUploader;
use observation_tools_common::artifact::ArtifactId;
pub use run_uploader::RunUploader;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

/// Updater for an Object2.
#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object2Updater {
    pub(crate) id: ArtifactId,
}
