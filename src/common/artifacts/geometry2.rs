use crate::artifacts::Image2;
use crate::artifacts::Point2;
use crate::artifacts::Polygon2;
use crate::artifacts::Rect2;
use crate::artifacts::Segment2;
use wasm_bindgen::prelude::*;

/// 2D geometry. Normally you do not need to interact with this type directly.
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum Geometry2 {
    Point2(Point2),
    Polygon2(Polygon2),
    Segment2(Segment2),
    Image2(Image2),
    Rect2(Rect2),
}

#[cfg(feature = "wasm")]
//#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Geometry2 | Point2 | Segment2 | Polygon2 | Image2 | Rect2")]
    pub type IntoGeometry2;
}
