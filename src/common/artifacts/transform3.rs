use crate::artifacts::transform3;
use crate::artifacts::Point3;
use crate::artifacts::Vector3;
use crate::math::Matrix4x4;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum Transform3 {
    Identity,
    Trs3(TRS3),
    Matrix(Matrix4x4),
}

#[derive(Debug, Clone)]
pub struct TRS3 {
    pub translation: Option<Point3>,
    pub scale: Option<Vector3>,
}

//#[wasm_bindgen]
impl Transform3 {
    pub fn identity() -> Transform3 {
        Transform3::Identity
    }
}
