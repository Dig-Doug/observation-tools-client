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
//! # use observation_tools_client::{ArtifactUploadHandle, ClientError};
//! # use observation_tools_client::groups::ArtifactUploader2d;
//! use observation_tools_client::artifacts::Point2Builder;
//!
//! # async fn create_artifact(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! // Create methods do some data validation and may return an error
//! let upload_handle = uploader.create_object2("my-point", Point2Builder::new(5.0, 3.0))?;
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
//! [crate::artifacts::SeriesBuilder] to keep track of updates over a dimension,
//! e.g. time.
//!
//! ```rust
//! # use observation_tools_client::{ArtifactUploadHandle, ClientError};
//! # use observation_tools_client::groups::ArtifactUploader2d;
//! use observation_tools_client::artifacts::Object2Builder;
//! use observation_tools_client::artifacts::Point2Builder;
//! use observation_tools_client::artifacts::SeriesBuilder;
//! use observation_tools_client::artifacts::SeriesPointBuilder;
//! use observation_tools_client::artifacts::Transform2Builder;
//!
//! # async fn update_artifact(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! # // TODO(doug): This series example is pretty messy
//! let mut series_builder = SeriesBuilder::new();
//! let algorithm_step_dimension_id = series_builder.add_dimension("algorithm_step");
//! let algorithm_series_id = uploader.series("grid_algorithm", series_builder)?;
//!
//! let handle = uploader.create_object2("my-point", Point2Builder::new(0.0, 0.0))?;
//!
//! for i in 0..10 {
//!     let mut object2: Object2Builder = Point2Builder::new(i as f64, 0.0).into();
//!     object2.add_transform(Transform2Builder::identity());
//!     object2.set_series_point(&SeriesPointBuilder::new(
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
pub use run_uploader::RunUploader;
