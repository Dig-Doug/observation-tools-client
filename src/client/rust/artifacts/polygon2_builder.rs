use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Point2Builder;
use crate::artifacts::PolygonEdge2Builder;
use crate::generated::Polygon2;
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 2D polygon. Polygon2s are represented as an edge-loop, so an edge will be
/// automatically created between the last and first vertex.
///
/// # Example
/// ```rust
/// use observation_tools_client::artifacts::Point2Builder;
/// use observation_tools_client::artifacts::Polygon2Builder;
/// use observation_tools_client::artifacts::PolygonEdge2Builder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), observation_tools_client::ClientError> {
///     tracing_subscriber::fmt::init();
///     let client = observation_tools_client::test_utils::create_doc_test_client()?;
///
///     // Set up a 2D group:
///     let run = client.create_run("create_polygon2")?;
///     let group2d = run.child_uploader_2d("polygon2_world")?;
///
///     // Basic usage:
///     let mut polygon = Polygon2Builder::new();
///     polygon.add_edge(PolygonEdge2Builder::new((0.0, 0.0)));
///     polygon.add_edge(PolygonEdge2Builder::new((1.0, 0.0)));
///     polygon.add_edge(PolygonEdge2Builder::new((0.0, 1.0)));
///     group2d.create_object2("my_polygon", polygon)?;
///
///     // Polygon from list of points:
///     let points = vec![
///         nalgebra::Point2::new(2.0, 2.0),
///         nalgebra::Point2::new(3.0, 2.0),
///         nalgebra::Point2::new(3.0, 3.0),
///         nalgebra::Point2::new(2.0, 3.0),
///     ];
///     group2d.create_object2(
///         "my_polygon_from_points",
///         Polygon2Builder::from_points(points),
///     )?;
///
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Polygon2Builder {
    pub(crate) proto: Polygon2,
}

#[wasm_bindgen]
impl Polygon2Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon2Builder {
        let proto = Polygon2::new();
        Polygon2Builder { proto }
    }

    /// Add a vertex to the polygon.
    pub fn add_edge(&mut self, edge: PolygonEdge2Builder) {
        self.proto.edges.push(edge.proto);
    }
}

impl Polygon2Builder {
    pub fn from_points<T: Into<Point2Builder>>(points: Vec<T>) -> Polygon2Builder {
        Polygon2Builder::from_edges(
            &points
                .into_iter()
                .map(|point| PolygonEdge2Builder::new(point.into()))
                .collect_vec(),
        )
    }

    pub fn from_edges(edges: &[PolygonEdge2Builder]) -> Polygon2Builder {
        let mut proto = Polygon2::new();
        proto.edges = edges.iter().map(|edge| edge.proto.clone()).collect();
        Polygon2Builder { proto }
    }
}

impl Into<Geometry2Builder> for Polygon2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::polygon(self)
    }
}

impl Into<Object2Builder> for Polygon2Builder {
    fn into(self) -> Object2Builder {
        let builder = Object2Builder::new(self.into());
        builder
    }
}
