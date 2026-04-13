use opencab::models::patients::{
  ActiveModel as PatientActiveModel, CreatePatientParams, Model as PatientModel,
};
use sea_orm::DatabaseConnection;

pub struct PatientFactory {
  first_name: String,
  last_name: String,
  ssn: String,
  email: String,
  address_line_1: String,
  address_zip_code: String,
  address_city: String,
}

impl Default for PatientFactory {
  fn default() -> Self {
    Self {
      first_name: "Alice".to_string(),
      last_name: "Dupont".to_string(),
      ssn: "1234567890123".to_string(),
      email: "patient@test.com".to_string(),
      address_line_1: "2 avenue des Champs".to_string(),
      address_zip_code: "75008".to_string(),
      address_city: "Paris".to_string(),
    }
  }
}

impl PatientFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn first_name(mut self, first_name: &str) -> Self {
    self.first_name = first_name.to_string();
    self
  }

  pub fn last_name(mut self, last_name: &str) -> Self {
    self.last_name = last_name.to_string();
    self
  }

  pub async fn create(self, db: &DatabaseConnection, user_id: i32) -> PatientModel {
    let params = CreatePatientParams {
      first_name: self.first_name,
      last_name: self.last_name,
      ssn: Some(self.ssn),
      address_line_1: self.address_line_1,
      address_zip_code: self.address_zip_code,
      address_city: self.address_city,
      email: Some(self.email),
    };

    PatientActiveModel::create(db, &params, user_id)
      .await
      .unwrap()
  }
}
