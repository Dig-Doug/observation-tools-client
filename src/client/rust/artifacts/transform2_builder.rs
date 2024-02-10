use crate::artifacts::NumberBuilder;
use crate::artifacts::Point2Builder;
use crate::artifacts::Vector2Builder;
use crate::generated::Transform2;
use crate::generated::TRS2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Transform2Builder {
    pub(crate) proto: Transform2,
}

#[wasm_bindgen]
impl Transform2Builder {
    pub fn identity() -> Transform2Builder {
        let mut proto = Transform2::new();
        proto.set_identity(true);
        Transform2Builder { proto }
    }

    pub fn trs(
        translation: &Point2Builder,
        rotation: &NumberBuilder,
        scale: &Vector2Builder,
    ) -> Transform2Builder {
        let mut trs = TRS2::new();
        trs.translation = Some(translation.proto.clone()).into();
        trs.rotation = Some(rotation.proto.clone()).into();
        trs.scale = Some(scale.proto.clone()).into();
        let mut proto = Transform2::new();
        *proto.mut_trs() = trs;
        Transform2Builder { proto }
    }
}

impl Transform2Builder {
    pub fn scale<S: Into<Vector2Builder>>(scale: S) -> Transform2Builder {
        let mut trs = TRS2::new();
        trs.scale = Some(scale.into().proto).into();
        let mut proto = Transform2::new();
        *proto.mut_trs() = trs;
        Transform2Builder { proto }
    }

    pub fn from_trs<T: Into<Point2Builder>, R: Into<NumberBuilder>, S: Into<Vector2Builder>>(
        translation: T,
        rotation: R,
        scale: S,
    ) -> Transform2Builder {
        Transform2Builder::trs(&translation.into(), &rotation.into(), &scale.into())
    }
}
