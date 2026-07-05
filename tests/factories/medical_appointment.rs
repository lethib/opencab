use chrono::NaiveDate;
use opencab::models::{
  _entities::{medical_appointments, sea_orm_active_enums::PaymentMethod},
  medical_appointments::Model as AppointmentModel,
};
use sea_orm::TransactionTrait;

pub struct AppointmentFactory {
  date: NaiveDate,
  price_in_cents: i32,
  payment_method: Option<PaymentMethod>,
}

impl Default for AppointmentFactory {
  fn default() -> Self {
    Self {
      date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
      price_in_cents: 5000,
      payment_method: None,
    }
  }
}

impl AppointmentFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn date(mut self, date: &str) -> Self {
    self.date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    self
  }

  pub fn price(mut self, price_in_cents: i32) -> Self {
    self.price_in_cents = price_in_cents;
    self
  }

  pub fn payment_method(mut self, method: PaymentMethod) -> Self {
    self.payment_method = Some(method);
    self
  }

  pub async fn create(self, db: &impl TransactionTrait, user_id: i32, patient_id: i32, office_id: i32) -> AppointmentModel {
    medical_appointments::ActiveModel::builder()
      .set_date(self.date)
      .set_practitioner_office_id(office_id)
      .set_price_in_cents(self.price_in_cents)
      .set_payment_method(self.payment_method)
      .set_patient_id(patient_id)
      .set_user_id(user_id)
      .insert(db)
      .await
      .unwrap()
      .into()
  }
}
