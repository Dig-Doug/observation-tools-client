use crate::artifacts::transform2;
use crate::artifacts::Number;
use crate::artifacts::Point2;
use crate::artifacts::Vector2;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum Transform2 {
    Trs2(TRS2),
    Identity,
}

#[derive(Debug, Clone)]
pub struct TRS2 {
    pub translation: Option<Point2>,
    pub rotation: Option<Number>,
    pub scale: Option<Vector2>,
}

//#[wasm_bindgen]
impl Transform2 {
    pub fn identity() -> Transform2 {
        Transform2::Identity
    }

    pub fn trs(translation: &Point2, rotation: &Number, scale: &Vector2) -> Transform2 {
        Transform2::Trs2(TRS2 {
            translation: Some(translation.clone()),
            rotation: Some(rotation.clone()),
            scale: Some(scale.clone()),
        })
    }
}

impl Transform2 {
    /// Create a transform for a translation.
    pub fn translation<T: Into<Point2>>(translation: T) -> Transform2 {
        Transform2::Trs2(TRS2 {
            translation: Some(translation.into()),
            rotation: None,
            scale: None,
        })
    }

    /// Create a transform for a scale.
    pub fn scale<S: Into<Vector2>>(scale: S) -> Transform2 {
        Transform2::Trs2(TRS2 {
            translation: None,
            rotation: None,
            scale: Some(scale.into()),
        })
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
