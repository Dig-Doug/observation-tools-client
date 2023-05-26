use crate::builders::number_builder::NumberOrNumberBuilder;
use crate::builders::Geometry2Builder;
use crate::builders::NumberBuilder;
use crate::util::ClientError;
use artifacts_api_rust_proto::Point2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Point2Builder {
    pub(crate) proto: Point2,
}

#[wasm_bindgen]
impl Point2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
    ) -> Result<Point2Builder, ClientError> {
        Ok(Point2Builder::from_number_builder(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
        ))
    }

    pub fn from_number_builder(x: NumberBuilder, y: NumberBuilder) -> Point2Builder {
        let mut proto = Point2::new();
        proto.x = Some(x.proto).into();
        proto.y = Some(y.proto).into();
        Point2Builder { proto }
    }
}

impl Point2Builder {
    pub fn new(x: impl Into<NumberBuilder>, y: impl Into<NumberBuilder>) -> Point2Builder {
        Point2Builder::from_number_builder(x.into(), y.into())
    }
}

impl Into<Geometry2Builder> for &Point2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::point(self)
    }
}