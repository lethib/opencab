use crate::{
  models::{
    _entities::{patients, practitioner_offices, users},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  },
  services::{
    invoice::{
      pdf::patient::{PatientInvoiceGenerator, PatientPdfArgs},
      Invoice, InvoiceKind,
    },
    storage::StorageService,
  },
};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GenerateInvoiceParams {
  pub amount: f32,
  pub date: String,
  pub office_id: i32,
}

pub async fn generate(
  patient: &patients::Model,
  params: &GenerateInvoiceParams,
  current_user: &users::Model,
  is_duplicate: bool,
  db: &DatabaseConnection,
) -> Result<Invoice, MyErrors> {
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
    .one(db)
    .await?
    .ok_or(UnexpectedError::should_not_happen())?;

  let business_info = current_user.business_information(db).await?;
  let decrypted_patient_ssn = patient.decrypt_ssn()?;

  let storage_service = match StorageService::new() {
    Ok(service) => Some(service),
    Err(e) => {
      tracing::warn!("Storage service unavailable: {}. Continuing without signature.", e);
      None
    }
  };

  let signature_data = match &storage_service {
    Some(service) => (service
      .fetch_signature(
        business_info
          .signature_file_name
          .as_ref()
          .ok_or(ApplicationError::unprocessable_entity("no_signature_filename"))?,
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

  Ok(Invoice {
    data: pdf_generator.to_bytes()?,
    filename,
    date: invoice_date,
    kind: InvoiceKind::Patient,
  })
}
