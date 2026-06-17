use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
  pub filename: String,
  pub content_type: &'static str,
  pub data: String,
}

impl EmailAttachment {
  pub fn from_bytes(filename: String, content_type: &'static str, data: &Vec<u8>) -> Self {
    Self {
      filename,
      content_type,
      data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),
    }
  }

  pub fn decode_data(&self) -> Result<Vec<u8>, base64::DecodeError> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &self.data)
  }
}
