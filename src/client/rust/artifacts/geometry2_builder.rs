use crate::artifacts::Image2Builder;
use crate::artifacts::Point2Builder;
use crate::artifacts::Polygon2Builder;
use crate::artifacts::Rect2Builder;
use crate::artifacts::Segment2Builder;
use crate::generated::Geometry2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// 2D geometry. Normally you do not need to interact with this type directly.
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone)]
pub struct Geometry2Builder {
    pub(crate) proto: Geometry2,
}

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "Geometry2Builder | Point2Builder | Segment2Builder | Polygon2Builder | Image2Builder | Rect2Builder"
    )]
    pub type IntoGeometry2Builder;
}

impl Geometry2Builder {
    pub(crate) fn point(point: Point2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_point2() = point.proto;
        Geometry2Builder { proto }
    }

    pub(crate) fn segment(segment: Segment2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_segment2() = segment.proto;
        Geometry2Builder { proto }
    }

    pub(crate) fn polygon(polygon: Polygon2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_polygon2() = polygon.proto;
        Geometry2Builder { proto }
    }

    pub(crate) fn image(image: Image2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_image2() = image.proto;
        Geometry2Builder { proto }
    }

    pub(crate) fn rect(rect: Rect2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_rect2() = rect.proto;
        Geometry2Builder { proto }
    }
}
