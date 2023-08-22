use crate::builders::Geometry3Builder;
use crate::builders::IntoGeometry3Builder;
use crate::builders::MeshBuilder;
use crate::builders::Polygon3Builder;
use crate::builders::SphereBuilder;
use crate::builders::Transform3Builder;
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
    #[wasm_bindgen(constructor)]
    pub fn new_js(value: IntoGeometry3Builder) -> Result<Object3Builder, ClientError> {
        let js_value: &JsValue = value.as_ref();
        if let Ok(val) = SphereBuilder::try_from(js_value) {
            return Ok((&val).into());
        }
        if let Ok(val) = Polygon3Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = MeshBuilder::try_from(js_value) {
            return Ok((&val).into());
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

impl Into<StructuredData> for &Object3Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_object3() = self.proto.clone();
        s
    }
}
