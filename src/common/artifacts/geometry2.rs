use crate::artifacts::Image2;
use crate::artifacts::Point2;
use crate::artifacts::Polygon2;
use crate::artifacts::Rect2;
use crate::artifacts::Segment2;
use serde::Deserialize;
use serde::Serialize;

/// 2D geometry. Normally you do not need to interact with this type directly.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Geometry2 {
    Point2(Point2),
    Polygon2(Polygon2),
    Segment2(Segment2),
    Image2(Image2),
    Rect2(Rect2),
}
