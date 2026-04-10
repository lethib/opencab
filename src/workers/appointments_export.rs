use crate::{
  db::DB,
  models::{
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
    users::users,
  },
  services::appointments::{MedicalAppointmentExtractor, ToExcel},
  workers::{WorkerJob, WorkerTransmitter},
  workers::mailer::{args::EmailArgs, attachment::EmailAttachment},
};
use chrono::NaiveDate;
use rust_xlsxwriter::Workbook;

#[derive(Debug, Clone)]
pub struct AppointmentExtractorArgs {
  pub user: users::Model,
  pub start_date: NaiveDate,
  pub end_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct AccountabilityGenerationArgs {
  pub user: users::Model,
  pub year: u16,
}

const EXCEL_CONTENT_TYPE: &str =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

pub async fn process_accountability_generation(
  args: AccountabilityGenerationArgs,
) -> Result<(), MyErrors> {
  let start_date =
    NaiveDate::from_ymd_opt(args.year as i32, 1, 1).ok_or(UnexpectedError::ShouldNotHappen)?;
  let end_date =
    NaiveDate::from_ymd_opt(args.year as i32, 12, 31).ok_or(UnexpectedError::ShouldNotHappen)?;

  let workbook = MedicalAppointmentExtractor::for_user(&args.user)
    .extract(DB::get(), start_date, end_date)
    .await?
    .generate_accountability()?;

  send_accountability_by_mail(workbook, args.user.email, args.year).await?;

  Ok(())
}

async fn send_accountability_by_mail(
  mut workbook: Workbook,
  to: String,
  year: u16,
) -> Result<(), MyErrors> {
  let wb_buffer = workbook.save_to_buffer()?;

  let workbook_attachment = EmailAttachment::from_bytes(
    format!("{}_accountability.xlsx", year),
    EXCEL_CONTENT_TYPE.to_string(),
    &wb_buffer,
  );

  let email_args = EmailArgs::new_text(
    to,
    format!("Votre comptabilité de l'année {}", year),
    format!(
      "Bonjour,\n\nVous trouverez votre comptabilité de l'année {} en pièce jointe.",
      year
    ),
  )
  .with_attachment(workbook_attachment);

  WorkerTransmitter::get()
    .send(WorkerJob::Email(email_args))
    .await
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  Ok(())
}

pub async fn process_appointment_extraction(
  args: AppointmentExtractorArgs,
) -> Result<(), MyErrors> {
  let workbook = MedicalAppointmentExtractor::for_user(&args.user)
    .extract(DB::get(), args.start_date, args.end_date)
    .await?
    .export_appointments()?;

  send_appointment_export_by_mail(workbook, args.user.email, args.start_date, args.end_date)
    .await?;

  Ok(())
}

async fn send_appointment_export_by_mail(
  mut workbook: Workbook,
  to: String,
  start_date: NaiveDate,
  end_date: NaiveDate,
) -> Result<(), MyErrors> {
  let wb_buffer = workbook.save_to_buffer()?;

  let workbook_attachment = EmailAttachment::from_bytes(
    format!(
      "appointments_from_{}_to_{}.xlsx",
      start_date.format("%d-%m-%Y"),
      end_date.format("%d-%m-%Y")
    ),
    EXCEL_CONTENT_TYPE.to_string(),
    &wb_buffer,
  );

  let email_args = EmailArgs::new_text(
    to,
    format!(
      "Vos RDV du {} au {}",
      start_date.format("%d/%m/%Y"),
      end_date.format("%d/%m/%Y")
    ),
    "Bonjour,\n\nVous trouverez tous vos rendez-vous de la période sélectionnée en pièce jointe"
      .to_string(),
  )
  .with_attachment(workbook_attachment);

  WorkerTransmitter::get()
    .send(WorkerJob::Email(email_args))
    .await
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  Ok(())
}
