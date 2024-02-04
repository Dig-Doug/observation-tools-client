use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use crate::generated::Vector2;
use crate::util::ClientError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Vector2Builder {
    pub(crate) proto: Vector2,
}

#[wasm_bindgen]
impl Vector2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
    ) -> Result<Vector2Builder, ClientError> {
        Ok(Vector2Builder::from_number_builder(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
        ))
    }

    pub fn from_number_builder(x: NumberBuilder, y: NumberBuilder) -> Vector2Builder {
        let mut proto = Vector2::new();
        proto.x = Some(x.proto).into();
        proto.y = Some(y.proto).into();
        Vector2Builder { proto }
    }
}

impl Vector2Builder {
    pub fn new(x: impl Into<NumberBuilder>, y: impl Into<NumberBuilder>) -> Vector2Builder {
        Vector2Builder::from_number_builder(x.into(), y.into())
    }
}
