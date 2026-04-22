use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub const TOKEN_TYPE_AUTH: &str = "auth";
pub const TOKEN_TYPE_PASSWORD_RESET: &str = "password_reset";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub pid: String,
  pub exp: i64,
  pub iat: i64,
  pub token_type: String,
}

pub struct JwtService {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
}

impl JwtService {
  pub fn new(secret: &str) -> Self {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 0;

    Self {
      encoding_key: EncodingKey::from_secret(secret.as_bytes()),
      decoding_key: DecodingKey::from_secret(secret.as_bytes()),
      validation,
    }
  }

  pub fn generate_token(
    &self,
    pid: &str,
    token_type: &str,
    expiration_seconds: u64,
  ) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::seconds(expiration_seconds as i64);

    let claims = Claims {
      pid: pid.to_string(),
      exp: exp.timestamp(),
      iat: now.timestamp(),
      token_type: token_type.to_string(),
    };

    encode(&Header::default(), &claims, &self.encoding_key)
  }

  pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
    Ok(token_data.claims)
  }
}

#[cfg(test)]
mod tests {
  use std::{sync::LazyLock, thread::sleep, time};

  use super::*;

  static JWT_SERVICE: LazyLock<JwtService> = LazyLock::new(|| JwtService::new("test_secret"));

  #[test]
  fn validate_token_can_be_decoded() {
    let token = JWT_SERVICE
      .generate_token("user-pid-123", TOKEN_TYPE_AUTH, 500)
      .unwrap();
    let claims = JWT_SERVICE.validate_token(&token).unwrap();
    assert_eq!(claims.pid, "user-pid-123");
    assert_eq!(claims.token_type, TOKEN_TYPE_AUTH);
  }

  #[test]
  fn modify_token_is_rejected() {
    let mut token = JWT_SERVICE
      .generate_token("user-pid-123", TOKEN_TYPE_AUTH, 500)
      .unwrap();

    token.push_str("foo");
    assert!(JWT_SERVICE.validate_token(&token).is_err());
  }

  #[test]
  fn expired_token_is_rejected() {
    let token = JWT_SERVICE
      .generate_token("user-pid-123", TOKEN_TYPE_AUTH, 1)
      .unwrap();
    sleep(time::Duration::from_secs(2));

    assert!(JWT_SERVICE.validate_token(&token).is_err())
  }
}
