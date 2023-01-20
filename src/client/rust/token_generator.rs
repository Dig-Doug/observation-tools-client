#[derive(Clone)]
pub enum TokenGenerator {
  Constant(String),
}

impl TokenGenerator {
  pub async fn token(&self) -> Result<String, std::io::Error> {
    match self {
      TokenGenerator::Constant(s) => Ok(s.clone())
    }
  }
}

