#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use observation_tools_common::proto::Vector3;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Vector3Builder {
    pub(crate) proto: Vector3,
}

#[wasm_bindgen]
impl Vector3Builder {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Vector3Builder, crate::ClientError> {
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
        Vector3Builder {
            proto: Vector3 {
                x: Some(x.proto),
                y: Some(y.proto),
                z: Some(z.proto),
            },
        }
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
