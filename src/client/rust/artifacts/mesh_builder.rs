use crate::artifacts::Geometry3Builder;
use crate::artifacts::Object3Builder;
use crate::artifacts::VertexBuilder;
use crate::generated::Mesh;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 3D mesh
///
/// # Example
/// ```rust
/// use observation_tools_client::artifacts::MeshBuilder;
/// use observation_tools_client::artifacts::Transform3Builder;
/// use observation_tools_client::artifacts::VertexBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), observation_tools_client::ClientError> {
///     tracing_subscriber::fmt::init();
///     let client = observation_tools_client::test_utils::create_doc_test_client()?;
///
///     // Set up a 3D group:
///     let run = client.create_run("create_mesh3")?;
///     let group3d = run.child_uploader_3d("mesh3_world")?;
///
///     // Basic usage:
///     let mut mesh = MeshBuilder::new();
///     mesh.add_vertex(VertexBuilder::new((0.0, 0.0, 0.0)));
///     mesh.add_vertex(VertexBuilder::new((1.0, 0.0, 0.0)));
///     mesh.add_vertex(VertexBuilder::new((0.0, 1.0, 0.0)));
///     mesh.add_triangle(0, 1, 2);
///     group3d.create_object3("my_mesh", mesh)?;
///
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct MeshBuilder {
    pub(crate) proto: Mesh,
}

#[wasm_bindgen]
impl MeshBuilder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> MeshBuilder {
        let proto = Mesh::new();
        MeshBuilder { proto }
    }

    pub fn add_vertex(&mut self, vertex: VertexBuilder) {
        self.proto.vertices.push(vertex.proto);
    }

    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.proto.indices.push(i0);
        self.proto.indices.push(i1);
        self.proto.indices.push(i2);
    }
}

impl Into<Geometry3Builder> for MeshBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::mesh(self)
    }
}

impl Into<Object3Builder> for MeshBuilder {
    fn into(self) -> Object3Builder {
        let builder = Object3Builder::new(self.into());
        builder
    }
}
