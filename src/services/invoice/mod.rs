use chrono::NaiveDate;

use crate::{
  models::{
    _entities::{practitioner_companies, sea_orm_active_enums::Profession},
    my_errors::MyErrors,
    users::users,
  },
  services::invoice::email::{send_company_invoice, send_patient_invoice},
};

pub mod company_invoice;
pub mod email;
pub mod patient_invoice;
mod pdf;

pub enum InvoiceKind {
  Patient,
  Company(Box<practitioner_companies::Model>),
}

pub struct Invoice {
  pub data: Vec<u8>,
  pub filename: String,
  pub date: NaiveDate,
  pub kind: InvoiceKind,
}

impl Invoice {
  pub async fn send_to(&self, email: &str, from: &users::Model, profession: &Profession) -> Result<(), MyErrors> {
    match &self.kind {
      InvoiceKind::Patient => send_patient_invoice(email, self, from, profession).await,
      InvoiceKind::Company(company) => send_company_invoice(email, self, from, profession, company).await,
    }
  }
}
