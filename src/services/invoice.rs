use crate::{
  db::DB,
  models::{
    _entities::{
      company_interventions, patients, practitioner_companies, practitioner_offices,
      user_business_informations, users,
    },
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  },
  services::storage::StorageService,
  workers::{
    self,
    invoice_generator::{CompanyInvoiceArgs, InvoiceGeneratorArgs},
    mailer::{args::EmailArgs, attachment::EmailAttachment},
    WorkerJob, WorkerTransmitter,
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
  invoice_date: chrono::NaiveDate,
}

pub async fn send_invoice(
  generated_invoice: &GenerateInvoiceResponse,
  current_user: &users::Model,
  user_business_informations: &user_business_informations::Model,
) -> Result<(), MyErrors> {
  if generated_invoice.patient_email.is_none() {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  let attachment = EmailAttachment::from_bytes(
    generated_invoice.filename.clone(),
    "application/pdf",
    &generated_invoice.pdf_data,
  );

  let invoice_date = generated_invoice
    .invoice_date
    .format("%d/%m/%Y")
    .to_string();

  let args = EmailArgs::new_text(
    generated_invoice
      .patient_email
      .clone()
      .expect("checked ahead"),
    format!("Note d'honoraires {}", invoice_date),
    format!(
      "Vous trouverez ci-joint votre facture pour la consultation du {}\n\n{} {}\n{}\n{}",
      invoice_date,
      current_user.last_name,
      current_user.first_name,
      user_business_informations.profession.to_french(),
      current_user.phone_number
    ),
  )
  .set_from_name(format!(
    "{} {}",
    current_user.first_name, current_user.last_name
  ))
  .with_attachment(attachment)
  .with_reply_to(current_user.email.to_string());

  // Enqueue email job via worker channel
  WorkerTransmitter::get()
    .send(WorkerJob::Email(args))
    .await
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  Ok(())
}

pub async fn generate_patient_invoice(
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

  let args = InvoiceGeneratorArgs {
    patient: patient.clone(),
    user: current_user.clone(),
    amount: params.amount,
    invoice_date,
    practitioner_office,
    is_duplicate,
  };

  let pdf_data = workers::invoice_generator::generate_invoice_pdf(&args).await?;

  Ok(GenerateInvoiceResponse {
    pdf_data,
    filename,
    patient_email: patient.email,
    invoice_date,
  })
}

pub async fn generate_company_invoice(
  company_intervention: &company_interventions::Model,
  current_user: &users::Model,
  practitioner_office: practitioner_offices::Model,
) -> Result<Vec<u8>, MyErrors> {
  let business_info = user_business_informations::Entity::find()
    .filter(user_business_informations::Column::UserId.eq(current_user.id))
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let company = practitioner_companies::Entity::find_by_id(company_intervention.company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let emission_date = chrono::Utc::now().date_naive();

  let signature_data = match StorageService::new() {
    Ok(service) => {
      if let Some(ref sig_name) = business_info.signature_file_name {
        match service.fetch_signature(sig_name).await {
          Ok(data) => Some(data),
          Err(e) => {
            tracing::warn!(
              "Failed to fetch signature for company invoice: {}. Continuing without.",
              e
            );
            None
          }
        }
      } else {
        None
      }
    }
    Err(e) => {
      tracing::warn!(
        "Storage service unavailable for company invoice: {}. Continuing without signature.",
        e
      );
      None
    }
  };

  let args = CompanyInvoiceArgs {
    intervention: company_intervention.clone(),
    user: current_user.clone(),
    business_info,
    company,
    emission_date,
    practitioner_office,
    signature_data,
  };

  workers::invoice_generator::generate_company_invoice_pdf(&args)
}
