use crate::artifacts::Geometry3Builder;
#[cfg(feature = "wasm")]
use crate::artifacts::IntoGeometry3Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Transform3Builder;
use crate::ClientError;
use observation_tools_common::proto::structured_data;
use observation_tools_common::proto::Object2;
use observation_tools_common::proto::Object3;
use observation_tools_common::proto::StructuredData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Object3Builder {
    pub(crate) proto: Object3,
}

#[wasm_bindgen]
impl Object3Builder {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(value: IntoGeometry3Builder) -> Result<Object3Builder, ClientError> {
        use crate::artifacts::MeshBuilder;
        use crate::artifacts::Polygon3Builder;
        use crate::artifacts::SphereBuilder;

        let js_value: &JsValue = value.as_ref();
        if let Ok(val) = SphereBuilder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Polygon3Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = MeshBuilder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Geometry3Builder::try_from(js_value) {
            return Ok(Object3Builder::new(val));
        }
        Err(ClientError::FailedToCreateGeometry3Builder)
    }

    pub fn add_transform(&mut self, transform: &Transform3Builder) {
        self.proto.transforms.push(transform.proto.clone());
    }
}

impl Object3Builder {
    pub fn new(geometry: Geometry3Builder) -> Object3Builder {
        Object3Builder {
            proto: Object3 {
                geometry: Some(geometry.proto),
                transforms: vec![],
            },
        }
    }
}

impl TryInto<StructuredData> for Object3Builder {
    type Error = ClientError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.proto.transforms.is_empty() {
            return Err(ClientError::NoTransformsInBuilder);
        }
        Ok(StructuredData {
            data: Some(structured_data::Data::Object3(self.proto)),
        })
    }
}
