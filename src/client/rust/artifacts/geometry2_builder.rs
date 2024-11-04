use crate::artifacts::Image2Builder;
use crate::artifacts::Point2Builder;
use crate::artifacts::Polygon2Builder;
use crate::artifacts::Rect2Builder;
use crate::artifacts::Segment2Builder;
use observation_tools_common::proto::geometry2;
use observation_tools_common::proto::Geometry2;
use wasm_bindgen::prelude::*;

/// 2D geometry. Normally you do not need to interact with this type directly.
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Geometry2Builder {
    pub(crate) proto: Geometry2,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "Geometry2Builder | Point2Builder | Segment2Builder | Polygon2Builder | Image2Builder | Rect2Builder"
    )]
    pub type IntoGeometry2Builder;
}

impl Geometry2Builder {
    pub(crate) fn point(point: Point2Builder) -> Geometry2Builder {
        Geometry2Builder {
            proto: Geometry2 {
                data: Some(geometry2::Data::Point2(point.proto)),
            },
        }
    }

    pub(crate) fn segment(segment: Segment2Builder) -> Geometry2Builder {
        Geometry2Builder {
            proto: Geometry2 {
                data: Some(geometry2::Data::Segment2(segment.proto)),
            },
        }
    }

    pub(crate) fn polygon(polygon: Polygon2Builder) -> Geometry2Builder {
        Geometry2Builder {
            proto: Geometry2 {
                data: Some(geometry2::Data::Polygon2(polygon.proto)),
            },
        }
    }

    pub(crate) fn image(image: Image2Builder) -> Geometry2Builder {
        Geometry2Builder {
            proto: Geometry2 {
                data: Some(geometry2::Data::Image2(image.proto)),
            },
        }
    }

    pub(crate) fn rect(rect: Rect2Builder) -> Geometry2Builder {
        Geometry2Builder {
            proto: Geometry2 {
                data: Some(geometry2::Data::Rect2(rect.proto)),
            },
        }
    }
}
