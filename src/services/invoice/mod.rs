use chrono::NaiveDate;

use crate::{
  models::{_entities::sea_orm_active_enums::Profession, my_errors::MyErrors, users::users},
  services::invoice::email::{send_company_invoice, send_patient_invoice},
};

pub mod company_invoice;
pub mod email;
pub mod patient_invoice;
mod pdf;

pub enum InvoiceKind {
  Patient,
  Company,
}

pub struct Invoice {
  pub data: Vec<u8>,
  pub filename: String,
  pub date: NaiveDate,
  pub kind: InvoiceKind,
}

impl Invoice {
  pub async fn send_to(&self, email: &str, from: &users::Model, profession: &Profession) -> Result<(), MyErrors> {
    match self.kind {
      InvoiceKind::Patient => send_patient_invoice(email, self, from, profession).await,
      InvoiceKind::Company => send_company_invoice().await,
    }
  }
}
