use crate::artifacts::Number;
use crate::artifacts::Point2;
use crate::artifacts::Vector2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform2 {
    #[wasm_bindgen(skip)]
    pub data: Transform2Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transform2Data {
    Trs2(TRS2),
    Identity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TRS2 {
    pub translation: Option<Point2>,
    pub rotation: Option<Number>,
    pub scale: Option<Vector2>,
}

#[wasm_bindgen]
impl Transform2 {
    pub fn identity() -> Transform2 {
        Transform2 {
            data: Transform2Data::Identity,
        }
    }

    pub fn trs(translation: &Point2, rotation: &Number, scale: &Vector2) -> Transform2 {
        Transform2 {
            data: Transform2Data::Trs2(TRS2 {
                translation: Some(translation.clone()),
                rotation: Some(rotation.clone()),
                scale: Some(scale.clone()),
            }),
        }
    }
}

impl Transform2 {
    /// Create a transform for a translation.
    pub fn translation<T: Into<Point2>>(translation: T) -> Transform2 {
        Transform2 {
            data: Transform2Data::Trs2(TRS2 {
                translation: Some(translation.into()),
                rotation: None,
                scale: None,
            }),
        }
    }

    /// Create a transform for a scale.
    pub fn scale<S: Into<Vector2>>(scale: S) -> Transform2 {
        Transform2 {
            data: Transform2Data::Trs2(TRS2 {
                translation: None,
                rotation: None,
                scale: Some(scale.into()),
            }),
        }
    }

    /// Create a transform from a translation, rotation (in radians), and scale.
    pub fn from_trs<T: Into<Point2>, R: Into<Number>, S: Into<Vector2>>(
        translation: T,
        rotation: R,
        scale: S,
    ) -> Transform2 {
        Transform2::trs(&translation.into(), &rotation.into(), &scale.into())
    }
}

impl Into<nalgebra::Transform3<f64>> for Transform2 {
    fn into(self) -> nalgebra::Transform3<f64> {
        match self.data {
            Transform2Data::Identity => nalgebra::Transform3::identity(),
            Transform2Data::Trs2(trs) => {
                let translation = trs
                    .translation
                    .map(|v| v.into())
                    .unwrap_or(nalgebra::Point2::origin());
                nalgebra::convert(nalgebra::Similarity::from_parts(
                    nalgebra::Translation::from(nalgebra::Point3::new(
                        translation[0],
                        translation[1],
                        0.0,
                    )),
                    nalgebra::Rotation::identity(),
                    1.0,
                ))
            }
        }
    }
}
