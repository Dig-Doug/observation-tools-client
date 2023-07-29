use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RunStageUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl RunStageUploader {
    /*
    pub fn child_uploader(&self, metadata: &UserMetadataBuilder) -> GenericArtifactUploader {
        self.base.child_uploader_old(metadata)
    }

    pub fn child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
        self.base.child_uploader_2d_old(metadata)
    }
     */
}
