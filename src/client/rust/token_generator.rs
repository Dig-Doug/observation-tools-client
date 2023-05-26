use crate::util::ClientError;

#[derive(Clone)]
pub enum TokenGenerator {
    Constant(String),
}

impl TokenGenerator {
    pub async fn token(&self) -> Result<String, ClientError> {
        match self {
            TokenGenerator::Constant(s) => Ok(s.clone()),
        }
    }
}