/// These functions are only exposed publicly due to [this issue](https://github.com/rust-lang/rust/issues/67295). Please do not use them.
use crate::Client;
/// These functions are only exposed publicly due to [this issue](https://github.com/rust-lang/rust/issues/67295). Please do not use them.
use crate::ClientError;
/// These functions are only exposed publicly due to [this issue](https://github.com/rust-lang/rust/issues/67295). Please do not use them.
use crate::ClientOptions;

#[doc(hidden)]
pub fn create_doc_test_client() -> Result<Client, ClientError> {
    let project_id = std::env::var("DOC_TEST_PROJECT_ID").map_err(ClientError::from_string)?;
    Client::new(
        project_id,
        ClientOptions {
            token_generator: crate::TokenGenerator::OAuth2BrowserFlow,
            ..Default::default()
        },
    )
}
