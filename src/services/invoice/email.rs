use crate::{
  models::{
    _entities::{practitioner_companies::Model, sea_orm_active_enums::Profession, users},
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
  },
  services::invoice::Invoice,
  workers::{
    mailer::{args::EmailArgs, attachment::EmailAttachment},
    WorkerJob, WorkerTransmitter,
  },
};

pub(super) async fn send_company_invoice(
  to: &str,
  generated_invoice: &Invoice,
  current_user: &users::Model,
  profession: &Profession,
  company: &Model,
) -> Result<(), MyErrors> {
  let attachment = EmailAttachment::from_bytes(
    generated_invoice.filename.to_string(),
    "application/pdf",
    &generated_invoice.data,
  );

  let invoice_date = generated_invoice.date.format("%d/%m/%Y").to_string();

  let args = EmailArgs::new_text(
    to.to_string(),
    format!("Facture {} {}", profession.to_french(), company.name),
    format!(
      "Bonjour,\n\nVous trouverez ci-joint la facture pour l'intervention du {}.\n\n Cordialement,\n{} {}\n{}\n{}",
      invoice_date,
      current_user.last_name,
      current_user.first_name,
      profession.to_french(),
      current_user.phone_number
    ),
  )
  .set_from_name(format!("{} {}", current_user.first_name, current_user.last_name))
  .with_attachment(attachment)
  .with_reply_to(current_user.email.to_string());

  WorkerTransmitter::get()
    .send(WorkerJob::Email(args))
    .await
    .map_err(|_| UnexpectedError::should_not_happen())?;

  Ok(())
}

pub(super) async fn send_patient_invoice(
  to: &str,
  generated_invoice: &Invoice,
  current_user: &users::Model,
  profession: &Profession,
) -> Result<(), MyErrors> {
  let attachment = EmailAttachment::from_bytes(
    generated_invoice.filename.to_string(),
    "application/pdf",
    &generated_invoice.data,
  );

  let invoice_date = generated_invoice.date.format("%d/%m/%Y").to_string();

  let args = EmailArgs::new_text(
    to.to_string(),
    format!("Note d'honoraires {}", invoice_date),
    format!(
      "Vous trouverez ci-joint votre facture pour la consultation du {}\n\n{} {}\n{}\n{}",
      invoice_date,
      current_user.last_name,
      current_user.first_name,
      profession.to_french(),
      current_user.phone_number
    ),
  )
  .set_from_name(format!("{} {}", current_user.first_name, current_user.last_name))
  .with_attachment(attachment)
  .with_reply_to(current_user.email.to_string());

  // Enqueue email job via worker channel
  WorkerTransmitter::get()
    .send(WorkerJob::Email(args))
    .await
    .map_err(|_| UnexpectedError::should_not_happen())?;

  Ok(())
}
