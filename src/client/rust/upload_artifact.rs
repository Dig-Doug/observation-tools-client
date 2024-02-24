use crate::generated::CreateArtifactRequest;
use crate::util::encode_id_proto;
use crate::util::GenericError;
use crate::ClientError;
use crate::TokenGenerator;
use base64::Engine;
use futures::future::BoxFuture;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use futures::TryFutureExt;
use protobuf::Message;
use reqwest::multipart::Part;
use std::task::Context;
use std::task::Poll;
use tower_service::Service;
use tracing::error;
use tracing::trace;

#[derive(Debug, Clone)]
pub(crate) struct UploadArtifactService {
    pub host: String,
    pub token_generator: TokenGenerator,
    pub client: reqwest::Client,
}

#[derive(Debug)]
pub struct UploadArtifactTask {
    pub request: CreateArtifactRequest,
    pub payload: Option<UploadArtifactTaskPayload>,
    pub completion_channel: async_channel::Sender<()>,
    #[cfg(not(feature = "tokio"))]
    pub(crate) shared_completion_channel: async_channel::Sender<()>,
}

impl UploadArtifactTask {
    pub fn artifact_id(&self) -> String {
        encode_id_proto(self.request.artifact_id.as_ref().unwrap_or_default())
    }
}

#[derive(Debug)]
pub enum UploadArtifactTaskPayload {
    #[cfg(feature = "files")]
    File(tempfile::NamedTempFile),
    #[cfg(not(feature = "files"))]
    Bytes(Vec<u8>),
}

impl Service<UploadArtifactTask> for UploadArtifactService {
    type Response = ();
    type Error = GenericError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, task: UploadArtifactTask) -> Self::Future {
        trace!("Setting up task for artifact: {}", task.artifact_id());

        #[cfg(feature = "tokio")]
        {
            Box::pin(upload_artifact_impl(
                self.client.clone(),
                self.token_generator.clone(),
                self.host.clone(),
                task,
            ))
        }
        #[cfg(not(feature = "tokio"))]
        {
            let token_generator = self.token_generator.clone();
            let client = self.client.clone();
            let host = self.host.clone();
            let (send_result, receive_result) = async_channel::bounded(1);
            wasm_bindgen_futures::spawn_local(async move {
                let result = crate::upload_artifact::upload_artifact_impl(
                    client,
                    token_generator,
                    host,
                    task,
                )
                .await;
                if let Err(e) = send_result.send(result).await {
                    error!("Failed to send result: {:?}", e);
                }
            });

            async move {
                let _ = receive_result.recv().await;
                Ok(())
            }
            .boxed()
        }
    }
}

async fn upload_artifact_impl(
    client: reqwest::Client,
    token_generator: TokenGenerator,
    host: String,
    task: UploadArtifactTask,
) -> Result<(), GenericError> {
    trace!("Uploading artifact: {:?}", task.artifact_id());

    let req_b64 = base64::engine::general_purpose::STANDARD.encode(task.request.write_to_bytes()?);
    let mut form = reqwest::multipart::Form::new().text("request", req_b64);
    if let Some(payload) = task.payload.as_ref() {
        let part = match payload {
            #[cfg(feature = "files")]
            UploadArtifactTaskPayload::File(f) => {
                use tokio_util::codec::BytesCodec;
                use tokio_util::codec::FramedRead;

                let file = tokio::fs::File::open(f).await?;
                Part::stream(reqwest::Body::wrap_stream(FramedRead::new(
                    file,
                    BytesCodec::new(),
                )))
            }
            #[cfg(not(feature = "files"))]
            UploadArtifactTaskPayload::Bytes(bytes) => Part::bytes(bytes.clone()),
        };
        form = form.part("raw_data", part);
    }

    let token = token_generator.token().await?;
    let response = client
        .post(format!("{}/create-artifact", host))
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| ClientError::GenericError {
            message: e.to_string(),
        })?;

    let status = response.status();

    #[cfg(not(feature = "tokio"))]
    if let Err(e) = task.shared_completion_channel.send(()).await {
        error!("Failed to send completion signal: {:?}", e);
    }

    if status.is_client_error() || status.is_server_error() {
        let response_body = response
            .text()
            .await
            .unwrap_or_else(|_| "No body".to_string());
        Err(ClientError::ServerError {
            status_code: status.as_u16(),
            response: response_body,
        })?
    } else {
        Ok(())
    }
}
