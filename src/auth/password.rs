use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
  Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
  let parsed_hash = match PasswordHash::new(hash) {
    Ok(h) => h,
    Err(_) => return false,
  };

  Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn correct_password_are_verified() {
    let hashed_password = hash_password("test123").unwrap();
    assert!(verify_password("test123", &hashed_password));
  }

  #[test]
  fn incorrect_password_are_not_verified() {
    let hashed_password = hash_password("test123").unwrap();
    assert!(!verify_password("123test", &hashed_password));
  }

  #[test]
  fn different_password_have_different_hash() {
    let hashed_password_1 = hash_password("test123").unwrap();
    let hashed_password_2 = hash_password("123test").unwrap();
    assert_ne!(hashed_password_1, hashed_password_2)
  }
}
