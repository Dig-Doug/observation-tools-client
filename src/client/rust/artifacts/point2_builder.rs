use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::Geometry2Builder;
use crate::artifacts::NumberBuilder;
use crate::artifacts::Object2Builder;
use crate::generated::Point2;
use crate::util::ClientError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
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

    pub fn origin() -> Point2Builder {
        Point2Builder::new(0.0, 0.0)
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

impl Into<Object2Builder> for Point2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::point(self))
    }
}
