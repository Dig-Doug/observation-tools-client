use crate::artifacts::Object2Updater;
use crate::artifacts::PublicSeriesId;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::groups::GenericArtifactUploader;
use crate::groups::RunUploader;
use crate::PublicArtifactId;
use async_channel::Receiver;
use async_trait::async_trait;
use std::ops::Deref;
use std::ops::DerefMut;
use wasm_bindgen::prelude::wasm_bindgen;

#[async_trait]
pub trait ArtifactUploadHandle<T>: Deref<Target = T> {
    async fn wait_for_upload(&self);
}

#[macro_export]
macro_rules! task_handle_impl {
    ($sub:ident $res:ident) => {
        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub struct $sub {
            #[wasm_bindgen(getter_with_clone)]
            pub result: $res,
            pub(crate) channel: async_channel::Receiver<()>,
        }

        #[wasm_bindgen]
        impl $sub {
            pub async fn wait_for(&self) {
                // TODO(doug): Expose error for caller
                let _unused = self.channel.recv().await;
            }
        }

        #[async_trait]
        impl ArtifactUploadHandle<$res> for $sub {
            async fn wait_for_upload(&self) {
                self.wait_for().await;
            }
        }

        impl Deref for $sub {
            type Target = $res;

            fn deref(&self) -> &Self::Target {
                &self.result
            }
        }

        impl DerefMut for $sub {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.result
            }
        }

        impl TaskHandle for $sub {
            type Result = $res;

            fn new(result: Self::Result, channel: Receiver<()>) -> Self {
                Self { result, channel }
            }

            fn args(self) -> (Self::Result, Receiver<()>) {
                (self.result, self.channel)
            }
        }
    };
}
// wasm-bindgen can't handle generics, so we have to implement each task handle
// manually
task_handle_impl!(PublicArtifactIdTaskHandle PublicArtifactId);
task_handle_impl!(GenericArtifactUploaderTaskHandle GenericArtifactUploader);
task_handle_impl!(ArtifactUploader2dTaskHandle ArtifactUploader2d);
task_handle_impl!(ArtifactUploader3dTaskHandle ArtifactUploader3d);
task_handle_impl!(PublicSeriesIdTaskHandle PublicSeriesId);
task_handle_impl!(RunUploaderTaskHandle RunUploader);
task_handle_impl!(Object2UpdaterTaskHandle Object2Updater);

#[derive(Debug, Clone)]
pub(crate) struct BaseArtifactUploaderTaskHandle {
    pub result: BaseArtifactUploader,
    channel: async_channel::Receiver<()>,
}

impl TaskHandle for BaseArtifactUploaderTaskHandle {
    type Result = BaseArtifactUploader;

    fn new(result: Self::Result, channel: Receiver<()>) -> Self {
        Self { result, channel }
    }

    fn args(self) -> (Self::Result, Receiver<()>) {
        (self.result, self.channel)
    }
}

pub(crate) trait TaskHandle: Sized {
    type Result;

    fn new(result: Self::Result, channel: async_channel::Receiver<()>) -> Self;

    fn args(self) -> (Self::Result, async_channel::Receiver<()>);

    fn map_handle<T: TaskHandle>(self, f: impl FnOnce(Self::Result) -> T::Result) -> T {
        let (result, channel) = self.args();
        T::new(f(result), channel)
    }
}
