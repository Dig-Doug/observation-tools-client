use crate::artifact::StructuredData;
use crate::artifacts::ArtifactError;
use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use image::GrayImage;
use image::ImageBuffer;
use image::ImageOutputFormat;
use serde::Deserialize;
use serde::Serialize;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// An image.
//#[doc = docify::embed_run!("tests/examples.rs", image2_example)]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image2 {
    #[wasm_bindgen(skip)]
    pub data: Vec<u8>,
    #[wasm_bindgen(skip)]
    pub mime_type: String,
    #[wasm_bindgen(skip)]
    pub pixel_transform: Option<PerPixelTransform>,
}

#[wasm_bindgen]
impl Image2 {
    pub fn new(data: &[u8], mime_type: &str) -> Self {
        Image2 {
            data: data.to_vec(),
            mime_type: mime_type.to_string(),
            pixel_transform: None,
        }
    }

    pub fn from_grayscale_pixels(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Image2, ArtifactError> {
        let img: GrayImage = ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or(ArtifactError::FailedToCreateImage)?;
        let mut proto = Image2 {
            data: vec![],
            mime_type: "image/png".to_string(),
            pixel_transform: None,
        };
        let mut cursor = Cursor::new(&mut proto.data);
        img.write_to(&mut cursor, ImageOutputFormat::Png)
            .map_err(|e| ArtifactError::FailedToWriteImage {
                message: e.to_string(),
            })?;
        Ok(proto)
    }

    pub fn set_per_pixel_transform(&mut self, transform: PerPixelTransform) {
        self.pixel_transform = Some(transform);
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Image2 {
    pub fn into_object(self) -> Object2 {
        self.into()
    }
}

impl Into<StructuredData> for Image2 {
    fn into(self) -> StructuredData {
        StructuredData::Image2(self)
    }
}

impl Into<Geometry2> for Image2 {
    fn into(self) -> Geometry2 {
        Geometry2::Image2(self)
    }
}

impl Into<Object2> for Image2 {
    fn into(self) -> Object2 {
        let builder = Object2::new(self.into());
        builder
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerPixelTransform {
    #[wasm_bindgen(skip)]
    pub data: PerPixelTransformData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PerPixelTransformData {
    RandomDistinctColor,
}

#[wasm_bindgen]
impl PerPixelTransform {
    pub fn random_distinct_color() -> Self {
        PerPixelTransform {
            data: PerPixelTransformData::RandomDistinctColor,
        }
    }
}
