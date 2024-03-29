use crate::artifacts::Geometry3Builder;
#[cfg(feature = "wasm")]
use crate::artifacts::IntoGeometry3Builder;
use crate::artifacts::Transform3Builder;
use crate::generated::Object3;
use crate::generated::StructuredData;
use crate::ClientError;
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
        let mut proto = Object3::new();
        proto.geometry = Some(geometry.proto).into();
        Object3Builder { proto }
    }
}

impl TryInto<StructuredData> for Object3Builder {
    type Error = ClientError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.proto.transforms.is_empty() {
            return Err(ClientError::NoTransformsInBuilder);
        }

        let mut s = StructuredData::new();
        *s.mut_object3() = self.proto;
        Ok(s)
    }
}
