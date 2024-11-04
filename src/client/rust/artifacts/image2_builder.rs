use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::util::ClientError;
use image::GrayImage;
use image::ImageBuffer;
use image::ImageOutputFormat;
use observation_tools_common::proto::structured_data;
use observation_tools_common::proto::Image2;
use observation_tools_common::proto::PerPixelTransform;
use observation_tools_common::proto::RandomDistinctColor;
use observation_tools_common::proto::StructuredData;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// An image.
#[doc = docify::embed_run!("tests/examples.rs", image2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[wasm_bindgen]
impl Image2Builder {
    pub fn new(data: &[u8], mime_type: &str) -> Self {
        Image2Builder {
            proto: Image2 {
                data: data.to_vec(),
                mime_type: mime_type.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn from_grayscale_pixels(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Image2Builder, ClientError> {
        let img: GrayImage = ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or(ClientError::FailedToCreateImage)?;
        let mut proto = Image2 {
            data: vec![],
            mime_type: "image/png".to_string(),
            ..Default::default()
        };
        let mut cursor = Cursor::new(&mut proto.data);
        img.write_to(&mut cursor, ImageOutputFormat::Png)
            .map_err(|e| ClientError::GenericError {
                message: e.to_string(),
            })?;
        Ok(Image2Builder { proto })
    }

    pub fn set_per_pixel_transform(&mut self, transform: PerPixelTransformBuilder) {
        self.proto.pixel_transform = Some(transform.proto).into();
    }
}

impl Into<StructuredData> for Image2Builder {
    fn into(self) -> StructuredData {
        StructuredData {
            data: Some(structured_data::Data::Image2(self.proto)),
        }
    }
}

impl Into<Geometry2Builder> for Image2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::image(self)
    }
}

impl Into<Object2Builder> for Image2Builder {
    fn into(self) -> Object2Builder {
        let builder = Object2Builder::new(self.into());
        builder
    }
}

#[wasm_bindgen]
pub struct PerPixelTransformBuilder {
    pub(crate) proto: PerPixelTransform,
}

#[wasm_bindgen]
impl PerPixelTransformBuilder {
    pub fn random_distinct_color() -> Self {
        PerPixelTransformBuilder {
            proto: PerPixelTransform {
                random_distinct_color: Some(RandomDistinctColor::default()).into(),
            },
        }
    }
}
