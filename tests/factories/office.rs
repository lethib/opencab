use opencab::models::{
  _entities::user_practitioner_offices,
  practitioner_offices::{ActiveModel as OfficeActiveModel, Model as OfficeModel, PractitionerOfficeParams},
  user_practitioner_offices::CreateLinkParams,
};
use sea_orm::{prelude::Decimal, DatabaseConnection};

pub struct OfficeFactory {
  name: String,
  address_line_1: String,
  address_zip_code: String,
  address_city: String,
}

impl Default for OfficeFactory {
  fn default() -> Self {
    Self {
      name: "Cabinet Central".to_string(),
      address_line_1: "1 rue de la Paix".to_string(),
      address_zip_code: "75001".to_string(),
      address_city: "Paris".to_string(),
    }
  }
}

impl OfficeFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }

  pub async fn create(self, db: &DatabaseConnection) -> OfficeModel {
    OfficeActiveModel::create(
      db,
      &PractitionerOfficeParams {
        name: self.name,
        address_line_1: self.address_line_1,
        address_zip_code: self.address_zip_code,
        address_city: self.address_city,
      },
    )
    .await
    .unwrap()
  }

  pub async fn create_for_user(
    self,
    db: &DatabaseConnection,
    user_id: i32,
    revenue_share: i64,
  ) -> OfficeModel {
    let office = self.create(db).await;
    user_practitioner_offices::ActiveModel::create(
      db,
      &CreateLinkParams {
        user_id,
        practitioner_office_id: office.id,
        revenue_share_percentage: Decimal::from(revenue_share),
      },
    )
    .await
    .unwrap();
    office
  }
}
