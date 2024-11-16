use crate::artifacts::Matrix4x4;
use crate::artifacts::Number;
use crate::artifacts::Point3;
use crate::artifacts::Vector3;
use nalgebra::RealField;
use nalgebra::Scalar;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform3 {
    #[wasm_bindgen(skip)]
    pub data: Transform3Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transform3Data {
    Identity,
    Trs3(TRS3),
    Matrix(Matrix4x4),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TRS3 {
    pub translation: Option<Point3>,
    pub scale: Option<Vector3>,
}

#[wasm_bindgen]
impl Transform3 {
    pub fn identity() -> Transform3 {
        Transform3 {
            data: Transform3Data::Identity,
        }
    }
}

impl<T: Scalar + RealField + Into<Number>> Into<Transform3> for nalgebra::Transform3<T> {
    fn into(self) -> Transform3 {
        Transform3 {
            data: Transform3Data::Matrix(self.matrix().into()),
        }
    }
}

impl Into<nalgebra::Transform3<f64>> for Transform3 {
    fn into(self) -> nalgebra::Transform3<f64> {
        match self.data {
            Transform3Data::Identity => nalgebra::Transform3::identity(),
            Transform3Data::Trs3(trs) => nalgebra::convert(nalgebra::Similarity::from_parts(
                nalgebra::Translation::from(
                    trs.translation
                        .map(|v| v.into())
                        .unwrap_or(nalgebra::Point3::origin()),
                ),
                nalgebra::Rotation::identity(),
                1.0,
            )),
            Transform3Data::Matrix(matrix) => {
                nalgebra::Transform3::from_matrix_unchecked(matrix.into())
            }
        }
    }
}
