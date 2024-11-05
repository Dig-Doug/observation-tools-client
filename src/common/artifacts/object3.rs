use crate::artifact::StructuredData;
use crate::artifacts::Geometry3;
#[cfg(feature = "wasm")]
use crate::artifacts::IntoGeometry3;
use crate::artifacts::Object2;
use crate::artifacts::Transform3;
use anyhow::anyhow;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Object3 {
    pub geometry: Geometry3,
    pub transforms: Vec<Transform3>,
}

//#[wasm_bindgen]
impl Object3 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(value: IntoGeometry3) -> Result<Object3, anyhow::Error> {
        use crate::artifacts::Mesh;
        use crate::artifacts::Polygon3;
        use crate::artifacts::Sphere;

        let js_value: &JsValue = value.as_ref();
        if let Ok(val) = Sphere::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Polygon3::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Mesh::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Geometry3::try_from(js_value) {
            return Ok(Object3::new(val));
        }
        Err(anyhow::Error::FailedToCreateGeometry3)
    }

    pub fn add_transform(&mut self, transform: Transform3) {
        self.transforms.push(transform);
    }
}

impl Object3 {
    pub fn new(geometry: Geometry3) -> Object3 {
        Object3 {
            geometry,
            transforms: vec![],
        }
    }
}

impl TryInto<StructuredData> for Object3 {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.transforms.is_empty() {
            return Err(anyhow!("No transforms in Object3"));
        }
        Ok(StructuredData::Object3(self))
    }
}
