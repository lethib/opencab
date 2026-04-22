use aes_gcm::{
  aead::{Aead, OsRng},
  AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
};
use argon2::{
  password_hash::{PasswordHasher, SaltString},
  Argon2, Params,
};
use base64::{engine::general_purpose as Base64Engine, Engine};

use crate::models::my_errors::{unexpected_error::UnexpectedError, MyErrors};

pub struct Crypto {
  pub encryption_key: Key<Aes256Gcm>,
}

impl Crypto {
  fn new() -> Result<Self, MyErrors> {
    let key_string = std::env::var("SSN_ENCRYPTION_KEY")?;

    if key_string.len() != 32 {
      return Err(UnexpectedError::ShouldNotHappen.into());
    }

    Ok(Crypto {
      encryption_key: *Key::<Aes256Gcm>::from_slice(key_string.as_bytes()),
    })
  }

  pub fn encrypt(str_to_encrypt: &str) -> Result<String, MyErrors> {
    let encryption_key = Self::new()?.encryption_key;
    let cipher = Aes256Gcm::new(&encryption_key);
    let nonce = Aes256Gcm::generate_nonce(OsRng);

    let encrypted_str = cipher
      .encrypt(&nonce, str_to_encrypt.as_bytes())
      .map_err(|err| UnexpectedError::new(err.to_string()))?;

    let mut final_encryption = nonce.to_vec();
    final_encryption.extend_from_slice(&encrypted_str);

    Ok(Base64Engine::STANDARD.encode(&final_encryption))
  }

  pub fn decrypt(encrypted_str: &str) -> Result<String, MyErrors> {
    let decryption_key = Self::new()?.encryption_key;
    let cipher = Aes256Gcm::new(&decryption_key);

    let encrypted_data = Base64Engine::STANDARD
      .decode(encrypted_str)
      .map_err(|err| UnexpectedError::new(err.to_string()))?;

    if encrypted_data.len() < 12 {
      return Err(UnexpectedError::ShouldNotHappen.into());
    }

    let (nonce_bytes, encrypted_data) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted_bytes = cipher
      .decrypt(nonce, encrypted_data)
      .map_err(|err| UnexpectedError::new(err.to_string()))?;

    String::from_utf8(decrypted_bytes).map_err(|err| UnexpectedError::new(err.to_string()).into())
  }

  pub fn hash(value: &str, salt: &String) -> Result<String, MyErrors> {
    let arg2 = Argon2::new(
      argon2::Algorithm::Argon2id,
      argon2::Version::V0x13,
      Params::default(),
    );

    let salt_string = SaltString::encode_b64(salt.as_bytes())
      .map_err(|err| UnexpectedError::new(err.to_string()))?;

    Ok(
      arg2
        .hash_password(value.as_bytes(), &salt_string)
        .map_err(|err| UnexpectedError::new(err.to_string()))?
        .to_string(),
    )
  }
}

#[cfg(test)]
mod tests {
  use base64::{engine::general_purpose::STANDARD, Engine};

  use super::*;

  fn setup() {
    unsafe {
      std::env::set_var("SSN_ENCRYPTION_KEY", "12345678901234567890123456789012");
      std::env::set_var("SSN_SALT_KEY", "bdd_test_salt_key_for_patients!!");
    }
  }

  mod encrypt {
    use super::*;

    mod when_encrypting_then_decrypting {
      use super::*;

      #[test]
      fn then_the_original_value_is_retrieved() {
        setup();
        let value = "1234567890123456";
        let encrypted = Crypto::encrypt(value).unwrap();
        let decrypted = Crypto::decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, value);
      }
    }

    mod when_encrypting_the_same_value_twice {
      use super::*;

      #[test]
      fn then_the_two_results_are_different() {
        setup();
        let value = "1234567890123456";
        let first = Crypto::encrypt(value).unwrap();
        let second = Crypto::encrypt(value).unwrap();
        assert_ne!(first, second);
      }
    }
  }

  mod decrypt {
    use super::*;

    mod when_the_input_is_not_valid_base64 {
      use super::*;

      #[test]
      fn then_decryption_fails() {
        setup();
        let result = Crypto::decrypt("this!is!not!valid!base64");
        assert!(result.is_err());
      }
    }

    mod when_the_encoded_data_is_too_short {
      use super::*;

      #[test]
      fn then_decryption_fails() {
        setup();
        let short_b64 = STANDARD.encode(b"short");
        let result = Crypto::decrypt(&short_b64);
        assert!(result.is_err());
      }
    }
  }

  mod hash {
    use super::*;

    mod when_hashing_the_same_value_with_the_same_salt {
      use super::*;

      #[test]
      fn then_the_two_hashes_are_identical() {
        setup();
        let value = "my_secret";
        let salt = "test_salt_1234".to_string();
        let first = Crypto::hash(value, &salt).unwrap();
        let second = Crypto::hash(value, &salt).unwrap();
        assert_eq!(first, second);
      }
    }

    mod when_hashing_a_value {
      use super::*;

      #[test]
      fn then_the_result_is_in_argon2_format() {
        setup();
        let salt = "test_salt_1234".to_string();
        let hash = Crypto::hash("my_secret", &salt).unwrap();
        assert!(hash.starts_with("$argon2"));
      }
    }
  }
}
