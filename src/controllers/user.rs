use crate::{
  middleware::context::Ctx,
  models::{
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
    user_business_informations::{BankingInformationParams, CreateBusinessInformation},
  },
  services::{self, storage::StorageService},
  views::practitioner_office::PractitionerOffice,
  workers::{appointments_export, WorkerJob, WorkerTransmitter},
};
use axum::{extract::Multipart, http::status, Json};
use chrono::{Datelike, NaiveDate, Utc};
use image::{imageops::FilterType, ImageFormat};
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExtractMedicalAppointmentsParams {
  start_date: String,
  end_date: String,
}

#[derive(Deserialize)]
pub struct GenerateAccountabilityParams {
  year: u16,
}

pub async fn save_business_info(
  ctx: Ctx,
  Json(business_information): Json<CreateBusinessInformation>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  services::user::save_business_information(&business_information, &ctx.current_user, &ctx.db).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn save_banking_info(
  ctx: Ctx,
  Json(banking_info): Json<BankingInformationParams>,
) -> Result<status::StatusCode, MyErrors> {
  let business_info = ctx
    .current_user
    .business_information(&ctx.db)
    .await
    .map_err(ApplicationError::unprocessable_entity)?
    .into_active_model();

  business_info.save_banking_information(&ctx.db, banking_info).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn my_offices(ctx: Ctx) -> Result<Json<Vec<PractitionerOffice>>, MyErrors> {
  let my_offices = ctx.current_user.get_my_offices(&ctx.db).await?;

  let serialized_offices: Vec<PractitionerOffice> = my_offices
    .iter()
    .map(|(office, upo)| PractitionerOffice::new_with_upo(office, upo))
    .collect();

  Ok(Json(serialized_offices))
}

pub async fn generate_accountability(
  ctx: Ctx,
  Json(params): Json<GenerateAccountabilityParams>,
) -> Result<status::StatusCode, MyErrors> {
  let current_year = Utc::now().year() as u16;
  if params.year < 2025 || params.year > current_year {
    return Err(ApplicationError::bad_request("year_outside_window").into());
  }

  let args = appointments_export::AccountabilityGenerationArgs {
    user: ctx.current_user,
    year: params.year,
  };

  WorkerTransmitter::get()
    .send(WorkerJob::AccountabilityGeneration(args, ctx.db))
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn extract_medical_appointments(
  ctx: Ctx,
  Json(params): Json<ExtractMedicalAppointmentsParams>,
) -> Result<status::StatusCode, MyErrors> {
  let start_date = NaiveDate::parse_from_str(params.start_date.as_str(), "%Y-%m-%d")?;
  let end_date = NaiveDate::parse_from_str(params.end_date.as_str(), "%Y-%m-%d")?;

  if start_date >= end_date {
    return Err(ApplicationError::bad_request("start_date_before_end_date").into());
  }

  let args = appointments_export::AppointmentExtractorArgs {
    user: ctx.current_user,
    start_date,
    end_date,
  };

  WorkerTransmitter::get()
    .send(WorkerJob::AppointmentExport(args, ctx.db))
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn get_signature_url(ctx: Ctx) -> Result<String, MyErrors> {
  let storage = StorageService::new()?;
  let user_bi = ctx.current_user.business_information(&ctx.db).await?;
  let signature_filename = user_bi.signature_file_name.ok_or(UnexpectedError::should_not_happen())?;

  Ok(storage.signature_url(&signature_filename))
}

pub async fn upload_signature(ctx: Ctx, mut multipart: Multipart) -> Result<status::StatusCode, MyErrors> {
  let field = multipart
    .next_field()
    .await
    .map_err(ApplicationError::bad_request)?
    .ok_or(UnexpectedError::should_not_happen())?;

  let field_name = field.name().ok_or(ApplicationError::bad_request("missing_name_field"))?;
  if field_name != "signature" {
    return Err(ApplicationError::bad_request("no_signature_field").into());
  }

  let signature_data = field.bytes().await.map_err(ApplicationError::unprocessable_entity)?;

  let img = image::load_from_memory(&signature_data).map_err(|e| {
    tracing::error!("Failed to load image: {}", e);
    ApplicationError::unprocessable_entity(e)
  })?;

  let resized = img.resize_exact(314, 156, FilterType::Lanczos3);

  let mut png_bytes: Vec<u8> = Vec::new();
  resized
    .write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)
    .map_err(|e| {
      tracing::error!("Failed to encode image: {}", e);
      ApplicationError::unprocessable_entity(e)
    })?;

  let filename = format!(
    "{}_{}_{}",
    &ctx.current_user.first_name.to_lowercase(),
    &ctx.current_user.last_name.to_lowercase(),
    &ctx.current_user.id.to_string()
  );

  let storage_service = services::storage::StorageService::new()?;
  storage_service.upload_signature(&png_bytes, &filename, "image/png").await?;

  let mut business_information = ctx.current_user.business_information(&ctx.db).await?.into_active_model();

  business_information.signature_file_name = ActiveValue::Set(Some(filename));
  business_information.update(&ctx.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}
