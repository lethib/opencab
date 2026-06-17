use crate::{
  db::DB,
  models::{
    _entities::{patients, practitioner_offices, users},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  },
  services::{
    invoice::pdf::patient::{PatientInvoiceGenerator, PatientPdfArgs},
    storage::StorageService,
  },
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GenerateInvoiceParams {
  pub amount: f32,
  pub date: String,
  pub office_id: i32,
}

pub struct GenerateInvoiceResponse {
  pub pdf_data: Vec<u8>,
  pub filename: String,
  pub patient_email: Option<String>,
  pub(super) invoice_date: chrono::NaiveDate,
}

pub async fn generate(
  patient_id: &i32,
  params: &GenerateInvoiceParams,
  current_user: &users::Model,
  is_duplicate: bool,
) -> Result<GenerateInvoiceResponse, MyErrors> {
  let patient = patients::Entity::find_by_id(*patient_id)
    .filter(patients::Column::UserId.eq(current_user.id))
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let invoice_date = chrono::NaiveDate::parse_from_str(&params.date, "%Y-%m-%d")?;

  let filename = format!(
    "{} {} Note d'honoraires - {} {} {}.pdf",
    current_user.first_name,
    current_user.last_name.to_uppercase(),
    &patient.last_name,
    &patient.first_name,
    invoice_date.format("%d_%m_%Y")
  );

  let practitioner_office = practitioner_offices::Entity::find_by_id(params.office_id)
    .one(DB::get())
    .await?
    .ok_or(UnexpectedError::ShouldNotHappen)?;

  let business_info = current_user.business_information(DB::get()).await?;
  let decrypted_patient_ssn = patient.decrypt_ssn()?;
  let patient_email = patient.email.clone();

  let storage_service = match StorageService::new() {
    Ok(service) => Some(service),
    Err(e) => {
      tracing::warn!(
        "Storage service unavailable: {}. Continuing without signature.",
        e
      );
      None
    }
  };

  let signature_data = match &storage_service {
    Some(service) => (service
      .fetch_signature(
        business_info
          .signature_file_name
          .as_ref()
          .ok_or(ApplicationError::UnprocessableEntity)?,
      )
      .await)
      .ok(),
    None => None,
  };

  let args = PatientPdfArgs {
    user: current_user.clone(),
    business_info,
    patient,
    decrypted_patient_ssn,
    amount: params.amount,
    date: invoice_date,
    office: practitioner_office,
    signature_data,
  };

  let mut pdf_generator = PatientInvoiceGenerator::new(args).build()?;

  if is_duplicate {
    pdf_generator = pdf_generator.with_duplicata()?;
  }

  Ok(GenerateInvoiceResponse {
    pdf_data: pdf_generator.to_bytes()?,
    filename,
    patient_email,
    invoice_date,
  })
}
