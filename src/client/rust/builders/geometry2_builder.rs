use crate::builders::{Image2Builder, Point2Builder, Polygon2Builder, Segment2Builder};
use artifacts_api_rust_proto::{Geometry2, Image2, StructuredData};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Geometry2Builder {
    pub(crate) proto: Geometry2,
}

// TODO(doug): These methods should probably consume the input
#[wasm_bindgen]
impl Geometry2Builder {
    pub fn point(point: &Point2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_point2() = point.proto.clone();
        Geometry2Builder { proto }
    }

    pub fn segment(segment: &Segment2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_segment2() = segment.proto.clone();
        Geometry2Builder { proto }
    }

    pub fn polygon(polygon: &Polygon2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_polygon2() = polygon.proto.clone();
        Geometry2Builder { proto }
    }

    pub fn image(image: &Image2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_image2() = image.proto.clone();
        Geometry2Builder { proto }
    }
}
