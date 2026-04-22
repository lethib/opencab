use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, IntoActiveModel};

use crate::models::{_entities::users, users::RegisterParams};

pub struct UserFactory {
  email: String,
  password: String,
  first_name: String,
  last_name: String,
  phone_number: String,
  is_access_key_verified: bool,
}

impl Default for UserFactory {
  fn default() -> Self {
    Self {
      email: "doctor@test.com".to_string(),
      password: "Test1234!".to_string(),
      first_name: "John".to_string(),
      last_name: "Doe".to_string(),
      phone_number: "0600000000".to_string(),
      is_access_key_verified: true,
    }
  }
}

impl UserFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_password(mut self, password: &str) -> Self {
    self.password = password.to_string();
    self
  }

  pub fn unverified(mut self) -> Self {
    self.is_access_key_verified = false;
    self
  }

  /// Creates a user in the real database. Verified by default; call `.unverified()` to skip.
  pub async fn create(self, db: &DatabaseConnection) -> users::Model {
    let is_verified = self.is_access_key_verified;

    let created = users::Model::create_with_password(
      db,
      &RegisterParams {
        email: self.email,
        password: self.password,
        first_name: self.first_name,
        last_name: self.last_name,
        phone_number: self.phone_number,
      },
    )
    .await
    .unwrap();

    if is_verified {
      let mut active = created.into_active_model();
      active.is_access_key_verified = ActiveValue::Set(true);
      active.update(db).await.unwrap()
    } else {
      created
    }
  }
}
