//! # Artifacts
//! Artifacts have a few major components:
//! 1. Geometry data, e.g. a mesh
//! 1. Transforms that define how the geometry is positioned
//! 1. Metadata
//!
//! To create an artifact within an [artifact group](crate::groups), you'll
//! ultimately need to create an **object builder** (e.g [Object2Builder]),
//! which contains the geometry and transforms, and a
//! [UserMetadataBuilder] to define the metadata. Once you have these pieces,
//! you can add the artifact to a [artifact group](crate::groups) using an
//! **artifact uploader**.
//!
//! ## Creating object builders
//! To create an object builder, you first need to represent your data as one of
//! our primitive types, e.g. a [Polygon2Builder]. Once you have it, there are a
//! couple ways to convert it into an object builder:
//! ### Direct creation
//! All primitives can be converted into an object builder directly using
//! `into()`. Then, you can add transforms to set the artifact's position:
//!
//! ```rust
//! use observation_tools_client::artifacts::Object2Builder;
//! use observation_tools_client::artifacts::Point2Builder;
//! use observation_tools_client::artifacts::Transform2Builder;
//!
//! let mut sphere: Object2Builder = Point2Builder::new(5.0, 3.0).into();
//! sphere.add_transform(Transform2Builder::identity());
//! ```
//!
//! ### Implicit creation
//! If you do not need to set an explicit transform, you skip the object builder
//! conversion:
//!
//! ```rust
//! # use observation_tools_client::ClientError;
//! # use observation_tools_client::groups::ArtifactUploader2d;
//! use observation_tools_client::artifacts::Point2Builder;
//!
//! # fn implicit_object_builder(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! // The Point2 will be converted to an Object2 with an identity transform
//! uploader.create_object2("my-point", Point2Builder::new(5.0, 3.0))?;
//! #   Ok(())
//! # }
//! ```
//!
//! ## Metadata
//! The [UserMetadataBuilder] is used to store the artifact's name and
//! additional metadata, stored as key-value pairs.
//!
//! If you do not need to add metadata, [UserMetadataBuilder] can be created
//! implicitly from a string:  
//! ```rust
//! # use observation_tools_client::ClientError;
//! # use observation_tools_client::groups::ArtifactUploader2d;
//! use observation_tools_client::artifacts::Point2Builder;
//! use observation_tools_client::artifacts::UserMetadataBuilder;
//!
//! # fn user_metadata(uploader: &ArtifactUploader2d) -> Result<(), ClientError>{
//! // Creates an artifact with the name "my-point"
//! uploader.create_object2("my-point", Point2Builder::new(5.0, 3.0))?;
//!
//! // Create an artifact with additional key-value data
//! let mut metadata = UserMetadataBuilder::new("point-with-metadata");
//! metadata.add_metadata("my-key", "my-value");
//! uploader.create_object2(metadata, Point2Builder::new(5.0, 3.0))?;
//! #   Ok(())
//! # }
//! ```
//!
//! ## Primitives
//! <div class="warning"> Support for 3D primitives is experimental </div>
//!
//! Don't see a primitive you're looking for? Need conversions from types in another crate? Let us know! File a [feature request](https://github.com/Dig-Doug/observation-tools-client/issues) or [email us](mailto:help@observation.tools).
mod geometry2_builder;
mod geometry3_builder;
mod image2_builder;
mod mesh_builder;
pub mod nalegbra;
mod number_builder;
mod object2_builder;
mod object3_builder;
mod point2_builder;
mod point3_builder;
mod polygon2_builder;
mod polygon3_builder;
mod polygon_edge2_builder;
mod polygon_edge3_builder;
mod rect2_builder;
mod segment2_builder;
mod series_builder;
mod sphere_builder;
mod transform2_builder;
mod transform3_builder;
mod user_metadata;
mod vector2_builder;
mod vector3_builder;
mod vertex_builder;

pub use geometry2_builder::Geometry2Builder;
pub use geometry2_builder::IntoGeometry2Builder;
pub use geometry3_builder::Geometry3Builder;
pub use geometry3_builder::IntoGeometry3Builder;
pub use image2_builder::Image2Builder;
pub use image2_builder::PerPixelTransformBuilder;
pub use mesh_builder::MeshBuilder;
pub use number_builder::NumberBuilder;
pub use number_builder::NumberOrNumberBuilder;
pub use object2_builder::Object2Builder;
pub use object2_builder::Object2Updater;
pub use object3_builder::Object3Builder;
pub use point2_builder::Point2Builder;
pub use point3_builder::Point3Builder;
pub use polygon2_builder::Polygon2Builder;
pub use polygon3_builder::Polygon3Builder;
pub use polygon_edge2_builder::PolygonEdge2Builder;
pub use polygon_edge3_builder::PolygonEdge3Builder;
pub use rect2_builder::Rect2Builder;
pub use segment2_builder::Segment2Builder;
pub use series_builder::PublicSeriesDimensionId;
pub use series_builder::PublicSeriesId;
pub use series_builder::SeriesBuilder;
pub use series_builder::SeriesDimensionBuilder;
pub use series_builder::SeriesPointBuilder;
pub use sphere_builder::SphereBuilder;
pub use transform2_builder::Transform2Builder;
pub use transform3_builder::Transform3Builder;
pub use user_metadata::UserMetadataBuilder;
pub use vector2_builder::Vector2Builder;
pub use vector3_builder::Vector3Builder;
pub use vertex_builder::VertexBuilder;
