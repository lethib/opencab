use crate::{
  app_state::{AppState, WorkerJob},
  initializers::get_services,
  models::{
    _entities::{
      patients, practitioner_offices::Entity as PractitionerOffices,
      sea_orm_active_enums::PaymentMethod, user_business_informations, users,
    },
    medical_appointments::{ActiveModel as MedicalAppointments, CreateMedicalAppointmentParams},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
    patients as PatientModel,
  },
  workers::{
    self,
    invoice_generator::InvoiceGeneratorArgs,
    mailer::{args::EmailArgs, attachment::EmailAttachment},
  },
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateInvoiceParams {
  pub amount: f32,
  pub invoice_date: String,
  pub should_be_sent_by_email: bool,
  pub practitioner_office_id: i32,
  pub payment_method: Option<PaymentMethod>,
}

pub struct GenerateInvoiceResponse {
  pub pdf_data: Vec<u8>,
  pub filename: String,
  patient_email: String,
  invoice_date: chrono::NaiveDate,
}

pub async fn send_invoice(
  state: &AppState,
  generated_invoice: &GenerateInvoiceResponse,
  current_user: &users::Model,
  user_business_informations: &user_business_informations::Model,
) -> Result<(), MyErrors> {
  if generated_invoice.patient_email == PatientModel::DEFAULT_EMAIL {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  let attachment = EmailAttachment::from_bytes(
    generated_invoice.filename.to_string(),
    "application/pdf".to_string(),
    &generated_invoice.pdf_data,
  );

  let invoice_date = generated_invoice
    .invoice_date
    .format("%d/%m/%Y")
    .to_string();

  let args = EmailArgs::new_text(
    generated_invoice.patient_email.clone(),
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
  state
    .worker_transmitter
    .send(WorkerJob::Email(args))
    .await
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  Ok(())
}

pub async fn generate_patient_invoice(
  patient_id: &i32,
  params: &GenerateInvoiceParams,
  current_user: &users::Model,
) -> Result<GenerateInvoiceResponse, MyErrors> {
  let services = get_services();

  let patient = patients::Entity::find_by_id(*patient_id)
    .filter(patients::Column::UserId.eq(current_user.id))
    .one(&services.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  if patient.email.is_none() {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  let invoice_date = chrono::NaiveDate::parse_from_str(&params.invoice_date, "%Y-%m-%d")?;

  let filename = format!(
    "{} {} Note d'honoraires - {} {} {}.pdf",
    current_user.first_name,
    current_user.last_name.to_uppercase(),
    &patient.last_name,
    &patient.first_name,
    invoice_date.format("%d_%m_%Y")
  );

  let medical_appointment_params = CreateMedicalAppointmentParams {
    user_id: current_user.id,
    patient_id: *patient_id,
    practitioner_office_id: params.practitioner_office_id,
    payment_method: params.payment_method.clone(),
    date: invoice_date,
    price_in_cents: (params.amount * 100.0).round() as i32,
  };

  let created_medical_appointment =
    MedicalAppointments::create(&services.db, &medical_appointment_params).await?;

  let practitioner_office =
    PractitionerOffices::find_by_id(created_medical_appointment.practitioner_office_id)
      .one(&services.db)
      .await?
      .ok_or(UnexpectedError::ShouldNotHappen)?;

  let args = InvoiceGeneratorArgs {
    patient: patient.clone(),
    user: current_user.clone(),
    amount: params.amount,
    invoice_date,
    practitioner_office,
  };

  let pdf_data = workers::invoice_generator::generate_invoice_pdf(&services.db, &args).await?;

  Ok(GenerateInvoiceResponse {
    pdf_data,
    filename,
    patient_email: patient.email.expect("checked ahead"),
    invoice_date,
  })
}
