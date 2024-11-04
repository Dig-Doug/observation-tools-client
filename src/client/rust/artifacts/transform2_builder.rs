use crate::artifacts::NumberBuilder;
use crate::artifacts::Point2Builder;
use crate::artifacts::Vector2Builder;
use observation_tools_common::proto::transform2;
use observation_tools_common::proto::Transform2;
use observation_tools_common::proto::Trs2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Transform2Builder {
    pub(crate) proto: Transform2,
}

#[wasm_bindgen]
impl Transform2Builder {
    pub fn identity() -> Transform2Builder {
        Transform2Builder {
            proto: Transform2 {
                data: Some(transform2::Data::Identity(true)),
            },
        }
    }

    pub fn trs(
        translation: &Point2Builder,
        rotation: &NumberBuilder,
        scale: &Vector2Builder,
    ) -> Transform2Builder {
        Transform2Builder {
            proto: Transform2 {
                data: Some(transform2::Data::Trs(Trs2 {
                    translation: Some(translation.proto.clone()),
                    rotation: Some(rotation.proto.clone()),
                    scale: Some(scale.proto.clone()),
                })),
            },
        }
    }
}

impl Transform2Builder {
    /// Create a transform for a translation.
    pub fn translation<T: Into<Point2Builder>>(translation: T) -> Transform2Builder {
        Transform2Builder {
            proto: Transform2 {
                data: Some(transform2::Data::Trs(Trs2 {
                    translation: Some(translation.into().proto),
                    rotation: None,
                    scale: None,
                })),
            },
        }
    }

    /// Create a transform for a scale.
    pub fn scale<S: Into<Vector2Builder>>(scale: S) -> Transform2Builder {
        Transform2Builder {
            proto: Transform2 {
                data: Some(transform2::Data::Trs(Trs2 {
                    translation: None,
                    rotation: None,
                    scale: Some(scale.into().proto),
                })),
            },
        }
    }

    /// Create a transform from a translation, rotation (in radians), and scale.
    pub fn from_trs<T: Into<Point2Builder>, R: Into<NumberBuilder>, S: Into<Vector2Builder>>(
        translation: T,
        rotation: R,
        scale: S,
    ) -> Transform2Builder {
        Transform2Builder::trs(&translation.into(), &rotation.into(), &scale.into())
    }
}
