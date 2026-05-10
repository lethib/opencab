use opencab::models::{_entities::practitioner_companies, practitioner_companies::CompanyParams};
use sea_orm::DatabaseConnection;

pub struct CompanyFactory {
  name: String,
  contact_name: String,
  contact_email: String,
}

impl Default for CompanyFactory {
  fn default() -> Self {
    Self {
      name: "Test Company".to_string(),
      contact_name: "John Doe".to_string(),
      contact_email: "company@test.com".to_string(),
    }
  }
}

impl CompanyFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn create_for_user(
    self,
    db: &DatabaseConnection,
    user_id: i32,
  ) -> practitioner_companies::Model {
    practitioner_companies::ActiveModel::create(
      db,
      user_id,
      &CompanyParams {
        name: self.name,
        contact_name: self.contact_name,
        contact_email: self.contact_email,
        siret: None,
        address_line_1: None,
        address_zip_code: None,
        address_city: None,
      },
    )
    .await
    .unwrap()
  }
}
