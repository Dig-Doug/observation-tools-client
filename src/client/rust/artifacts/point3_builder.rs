#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use crate::generated::Point3;
use wasm_bindgen::prelude::*;

/// A 3D point.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Point3Builder {
    pub(crate) proto: Point3,
}

#[wasm_bindgen]
impl Point3Builder {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Point3Builder, crate::ClientError> {
        Ok(Point3Builder::from_number_builder(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
            NumberBuilder::from_js_value(z)?,
        ))
    }

    pub fn from_number_builder(
        x: NumberBuilder,
        y: NumberBuilder,
        z: NumberBuilder,
    ) -> Point3Builder {
        let mut proto = Point3::new();
        proto.x = Some(x.proto).into();
        proto.y = Some(y.proto).into();
        proto.z = Some(z.proto).into();
        Point3Builder { proto }
    }
}

impl Point3Builder {
    pub fn new(
        x: impl Into<NumberBuilder>,
        y: impl Into<NumberBuilder>,
        z: impl Into<NumberBuilder>,
    ) -> Point3Builder {
        Point3Builder::from_number_builder(x.into(), y.into(), z.into())
    }
}

impl<A, B, C> From<(A, B, C)> for Point3Builder
where
    A: Into<NumberBuilder>,
    B: Into<NumberBuilder>,
    C: Into<NumberBuilder>,
{
    fn from((x, y, z): (A, B, C)) -> Point3Builder {
        Point3Builder::new(x, y, z)
    }
}
