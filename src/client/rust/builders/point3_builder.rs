use crate::builders::number_builder::NumberOrNumberBuilder;
use crate::builders::NumberBuilder;
use crate::util::{ClientError};
use artifacts_api_rust_proto::Point3;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Point3Builder {
    pub(crate) proto: Point3,
}

#[wasm_bindgen]
impl Point3Builder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Point3Builder, ClientError> {
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
