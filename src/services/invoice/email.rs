use crate::{
  models::{
    _entities::{user_business_informations, users},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  },
  services::invoice::patient_invoice::GenerateInvoiceResponse,
  workers::{
    mailer::{args::EmailArgs, attachment::EmailAttachment},
    WorkerJob, WorkerTransmitter,
  },
};

pub async fn send_invoice(
  generated_invoice: &GenerateInvoiceResponse,
  current_user: &users::Model,
  user_business_informations: &user_business_informations::Model,
) -> Result<(), MyErrors> {
  if generated_invoice.patient_email.is_none() {
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
