use crate::task_loop::UploadArtifactTask;
use crate::task_loop::UploadArtifactTaskPayload;
use crate::util::GenericError;
use crate::ClientError;
use crate::TokenGenerator;
use base64::Engine;
use futures::future::BoxFuture;
use protobuf::Message;
use reqwest::multipart::Part;
use std::task::Context;
use std::task::Poll;
use tower_service::Service;
use tracing::trace;

#[derive(Clone)]
pub(crate) struct UploadArtifactService {
    pub host: String,
    pub token_generator: TokenGenerator,
    pub client: reqwest::Client,
}

impl Service<UploadArtifactTask> for UploadArtifactService {
    type Response = ();
    type Error = GenericError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, task: UploadArtifactTask) -> Self::Future {
        let token_generator = self.token_generator.clone();
        let client = self.client.clone();
        let host = self.host.clone();
        Box::pin(async move {
            trace!("Handling artifact: {:?}", task.request);

            let req_b64 =
                base64::engine::general_purpose::STANDARD.encode(task.request.write_to_bytes()?);
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
        })
    }
}
