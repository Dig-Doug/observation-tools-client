use crate::builders::Image2Builder;
use crate::builders::Point2Builder;
use crate::builders::Polygon2Builder;
use crate::builders::Rect2Builder;
use crate::builders::Segment2Builder;
use crate::generated::Geometry2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Geometry2Builder {
    pub(crate) proto: Geometry2,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "Geometry2Builder | Point2Builder | Segment2Builder | Polygon2Builder | Image2Builder | Rect2Builder"
    )]
    pub type IntoGeometry2Builder;
}

// TODO(doug): These methods should probably consume the input
impl Geometry2Builder {
    pub(crate) fn point(point: &Point2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_point2() = point.proto.clone();
        Geometry2Builder { proto }
    }

    pub(crate) fn segment(segment: &Segment2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_segment2() = segment.proto.clone();
        Geometry2Builder { proto }
    }

    pub(crate) fn polygon(polygon: &Polygon2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_polygon2() = polygon.proto.clone();
        Geometry2Builder { proto }
    }

    pub(crate) fn image(image: &Image2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_image2() = image.proto.clone();
        Geometry2Builder { proto }
    }

    pub(crate) fn rect(rect: &Rect2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_rect2() = rect.proto.clone();
        Geometry2Builder { proto }
    }
}
