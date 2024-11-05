use crate::artifact::StructuredData;
use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use anyhow::anyhow;
use image::GrayImage;
use image::ImageBuffer;
use image::ImageOutputFormat;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// An image.
//#[doc = docify::embed_run!("tests/examples.rs", image2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Image2 {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub pixel_transform: Option<PerPixelTransform>,
}

//#[wasm_bindgen]
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
    ) -> Result<Image2, anyhow::Error> {
        let img: GrayImage = ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or(anyhow!("Failed to create raw image"))?;
        let mut proto = Image2 {
            data: vec![],
            mime_type: "image/png".to_string(),
            pixel_transform: None,
        };
        let mut cursor = Cursor::new(&mut proto.data);
        img.write_to(&mut cursor, ImageOutputFormat::Png)?;
        Ok(proto)
    }

    pub fn set_per_pixel_transform(&mut self, transform: PerPixelTransform) {
        self.pixel_transform = Some(transform);
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

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum PerPixelTransform {
    RandomDistinctColor,
}

//#[wasm_bindgen]
impl PerPixelTransform {
    pub fn random_distinct_color() -> Self {
        PerPixelTransform::RandomDistinctColor
    }
}
