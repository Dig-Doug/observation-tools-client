//! Artifacts have a few major components:
//! 1. Geometry data, e.g. a mesh
//! 1. Transforms that define how the geometry is positioned
//! 1. Metadata, e.g. the artifact's name
//!
//! To create an artifact within an [artifact group](crate::groups), you'll
//! ultimately need to create an **object builder** (e.g. [Object2]),
//! which contains the geometry and transforms, and a
//! [UserMetadata] to define the metadata. Once you have these pieces,
//! you can add the artifact to a [artifact group](crate::groups) using an
//! **artifact uploader**.
//!
//! # Creating object builders
//! To create an object builder, you first need to represent your data as one of
//! our primitive types, e.g. a [Polygon2]. Once you have it, there are a
//! couple ways to convert it into an object builder:
//! ## Direct creation
//! All primitives can be converted into an object builder directly using
//! `into()`. Then, you can add transforms to set the artifact's position:
//!
//! ```rust
//! use observation_tools::artifacts::Object2;
//! use observation_tools::artifacts::Point2;
//! use observation_tools::artifacts::Transform2;
//!
//! // Create a primitive Point2
//! let point = Point2::new(5.0, 3.0);
//! // Convert it into an Object2
//! let mut point: Object2 = point.into();
//! // Add an identity transform so the point is positioned at (5, 3)
//! point.add_transform(Transform2::identity());
//! ```
//!
//! If you want to display the same geometry at different positions, you can
//! also add multiple transforms to the artifact.
//!
//! ## Implicit creation
//! If you do not need to set an explicit transform, you skip the object builder
//! conversion:
//!
//! ```rust
//! # use observation_tools::anyhow::Error;
//! # use observation_tools::groups::ArtifactUploader2d;
//! use observation_tools::artifacts::Point2;
//!
//! # fn implicit_object(uploader: &ArtifactUploader2d) -> Result<(), anyhow::Error>{
//! // The Point2 will be converted to an Object2 with an identity transform
//! uploader.create_object2("my-point", Point2::new(5.0, 3.0))?;
//! #   Ok(())
//! # }
//! ```
//!
//! # Metadata
//! The [UserMetadata] is used to store the artifact's name and
//! additional metadata, stored as key-value pairs.
//!
//! If you do not need to add metadata, [UserMetadata] can be created
//! implicitly from a string:
//! ```rust
//! # use observation_tools::anyhow::Error;
//! # use observation_tools::groups::ArtifactUploader2d;
//! use observation_tools::artifacts::Point2;
//! use observation_tools::artifacts::UserMetadata;
//!
//! # fn user_metadata(uploader: &ArtifactUploader2d) -> Result<(), anyhow::Error>{
//! // Creates an artifact with the name "my-point"
//! uploader.create_object2("my-point", Point2::new(5.0, 3.0))?;
//!
//! // Create an artifact with additional key-value data
//! let mut metadata = UserMetadata::new("point-with-metadata");
//! metadata.add_metadata("my-key", "my-value");
//! uploader.create_object2(metadata, Point2::new(5.0, 3.0))?;
//! #   Ok(())
//! # }
//! ```
//!
//! # Primitives
//! Don't see a primitive you're looking for? Need conversions from types in another crate? Let us know! File a [feature request](https://github.com/Dig-Doug/observation-tools-client/issues) or [email us](mailto:help@observation.tools).
//!
//! <div class="warning"> Support for 3D primitives is experimental </div>
mod geometry2;
mod geometry3;
mod image2;
mod mesh;
//pub mod nalegbra;
mod number;
mod object2;
mod object3;
mod point2;
mod point3;
mod polygon2;
mod polygon3;
mod polygon_edge2;
mod polygon_edge3;
mod rect2;
mod segment2;
mod series;
mod sphere;
mod transform2;
mod transform3;
mod user_metadata;
mod vector2;
mod vector3;
mod vertex;

pub use geometry2::Geometry2;
//#[cfg(feature = "wasm")]
//pub use geometry2::IntoGeometry2;
pub use geometry3::Geometry3;
//#[cfg(feature = "wasm")]
//pub use geometry3::IntoGeometry3;
pub use image2::Image2;
pub use image2::PerPixelTransform;
pub use mesh::Mesh;
pub use number::Number;
//#[cfg(feature = "wasm")]
//pub use number::NumberOrNumber;
pub use object2::Object2;
pub use object2::Object2Updater;
pub use object3::Object3;
pub use point2::Point2;
pub use point3::Point3;
pub use polygon2::Polygon2;
pub use polygon3::Polygon3;
pub use polygon_edge2::PolygonEdge2;
pub use polygon_edge3::PolygonEdge3;
pub use rect2::Rect2;
pub use segment2::Segment2;
pub use series::Series;
pub use series::SeriesDimension;
pub use series::SeriesDimensionId;
pub use series::SeriesId;
pub use series::SeriesPoint;
pub use sphere::Sphere;
pub use transform2::Transform2;
pub use transform3::Transform3;
pub use user_metadata::UserMetadata;
pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vertex::Vertex;
