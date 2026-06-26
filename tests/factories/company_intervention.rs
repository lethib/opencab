use chrono::NaiveDate;
use opencab::models::{_entities::company_interventions, company_interventions::InterventionParams};
use rust_decimal::Decimal;
use sea_orm::ConnectionTrait;

pub struct CompanyInterventionFactory {
  quantity: i32,
  unit_price: f32,
  vat_rate: Decimal,
  issue_date: NaiveDate,
  object: String,
}

impl Default for CompanyInterventionFactory {
  fn default() -> Self {
    Self {
      quantity: 1,
      unit_price: 100.0,
      vat_rate: Decimal::new(200, 1), // 20.0
      issue_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
      object: "Test intervention".to_string(),
    }
  }
}

impl CompanyInterventionFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn create(self, db: &impl ConnectionTrait, practitioner_id: i32, company_id: i32) -> company_interventions::Model {
    company_interventions::ActiveModel::create(
      db,
      practitioner_id,
      company_id,
      &InterventionParams {
        quantity: self.quantity,
        unit_price: self.unit_price,
        vat_rate: self.vat_rate,
        issue_date: self.issue_date,
        object: self.object,
      },
    )
    .await
    .unwrap()
  }
}
