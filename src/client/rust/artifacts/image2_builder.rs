use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::generated::Image2;
use crate::generated::PerPixelTransform;
use crate::generated::RandomDistinctColor;
use crate::generated::StructuredData;
use crate::util::ClientError;
use image::GrayImage;
use image::ImageBuffer;
use image::ImageOutputFormat;
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// An image.
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone)]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Image2Builder {
    pub fn new(data: &[u8], mime_type: &str) -> Self {
        let mut proto = Image2::new();
        proto.data = data.to_vec();
        proto.mime_type = mime_type.to_string();
        Image2Builder { proto }
    }

    pub fn from_grayscale_pixels(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Image2Builder, ClientError> {
        let img: GrayImage = ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or(ClientError::FailedToCreateImage)?;
        let mut proto = Image2::new();
        let mut cursor = Cursor::new(&mut proto.data);
        img.write_to(&mut cursor, ImageOutputFormat::Png)
            .map_err(|e| ClientError::GenericError {
                message: e.to_string(),
            })?;
        proto.mime_type = "image/png".to_string();
        Ok(Image2Builder { proto })
    }

    pub fn set_per_pixel_transform(&mut self, transform: PerPixelTransformBuilder) {
        self.proto.pixel_transform = Some(transform.proto).into();
    }
}

impl Into<StructuredData> for Image2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_image2() = self.proto;
        s
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

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct PerPixelTransformBuilder {
    pub(crate) proto: PerPixelTransform,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl PerPixelTransformBuilder {
    pub fn random_distinct_color() -> Self {
        let mut proto = PerPixelTransform::new();
        proto.random_distinct_color = Some(RandomDistinctColor::new()).into();
        PerPixelTransformBuilder { proto }
    }
}
