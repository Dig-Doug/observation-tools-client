use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use crate::generated::Vector3;
use crate::util::ClientError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Vector3Builder {
    pub(crate) proto: Vector3,
}

#[wasm_bindgen]
impl Vector3Builder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Vector3Builder, ClientError> {
        Ok(Vector3Builder::from_number_builder(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
            NumberBuilder::from_js_value(z)?,
        ))
    }

    pub fn from_number_builder(
        x: NumberBuilder,
        y: NumberBuilder,
        z: NumberBuilder,
    ) -> Vector3Builder {
        let mut proto = Vector3::new();
        proto.x = Some(x.proto).into();
        proto.y = Some(y.proto).into();
        proto.z = Some(z.proto).into();
        Vector3Builder { proto }
    }
}

impl Vector3Builder {
    pub fn new(
        x: impl Into<NumberBuilder>,
        y: impl Into<NumberBuilder>,
        z: impl Into<NumberBuilder>,
    ) -> Vector3Builder {
        Vector3Builder::from_number_builder(x.into(), y.into(), z.into())
    }
}
