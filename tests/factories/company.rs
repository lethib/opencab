use opencab::{
  models::_entities::practitioner_companies,
  services::practitioner_companies::{CompanyParams, PractitionerCompaniesService},
};
use sea_orm::TransactionTrait;

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

  pub async fn create_for_user(self, db: &impl TransactionTrait, user_id: i32) -> practitioner_companies::Model {
    PractitionerCompaniesService::create(CompanyParams {
      name: self.name,
      contact_name: self.contact_name,
      contact_email: self.contact_email,
      siret: None,
      address_line_1: None,
      address_zip_code: None,
      address_city: None,
    })
    .unwrap()
    .for_user(user_id)
    .build(db)
    .await
    .unwrap()
  }
}
