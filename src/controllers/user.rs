use crate::{
  db::DB,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::prelude::UserBusinessInformations,
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
    user_business_informations::CreateBusinessInformation,
  },
  services::{self, storage::StorageService},
  views::practitioner_office::PractitionerOffice,
  workers::appointments_export,
  workers::{WorkerJob, WorkerTransmitter},
};
use axum::{debug_handler, extract::Multipart, http::status, Json};
use chrono::{Datelike, NaiveDate, Utc};
use image::{imageops::FilterType, ImageFormat};
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, ModelTrait};
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

#[debug_handler]
pub async fn save_business_info(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(business_information): Json<CreateBusinessInformation>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  services::user::save_business_information(&business_information, &current_user).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

#[debug_handler]
pub async fn my_offices(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
) -> Result<Json<Vec<PractitionerOffice>>, MyErrors> {
  let my_offices = current_user.get_my_offices(DB::get()).await?;

  let serialized_offices: Vec<PractitionerOffice> = my_offices
    .iter()
    .map(|(office, upo)| PractitionerOffice::new_with_upo(office, upo))
    .collect();

  Ok(Json(serialized_offices))
}

#[debug_handler]
pub async fn generate_accountability(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(params): Json<GenerateAccountabilityParams>,
) -> Result<status::StatusCode, MyErrors> {
  let current_year = Utc::now().year() as u16;
  if params.year < 2025 || params.year > current_year {
    return Err(ApplicationError::BadRequest.into());
  }

  let args = appointments_export::AccountabilityGenerationArgs {
    user: current_user,
    year: params.year,
  };

  WorkerTransmitter::get()
    .send(WorkerJob::AccountabilityGeneration(args))
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn extract_medical_appointments(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(params): Json<ExtractMedicalAppointmentsParams>,
) -> Result<status::StatusCode, MyErrors> {
  let start_date = NaiveDate::parse_from_str(params.start_date.as_str(), "%Y-%m-%d")?;
  let end_date = NaiveDate::parse_from_str(params.end_date.as_str(), "%Y-%m-%d")?;

  if start_date >= end_date {
    return Err(ApplicationError::new("start_date_before_end_date").into());
  }

  let args = appointments_export::AppointmentExtractorArgs {
    user: current_user,
    start_date,
    end_date,
  };

  WorkerTransmitter::get()
    .send(WorkerJob::AppointmentExport(args))
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn get_signature_url(
  AuthenticatedUser(_current_user, user_bi): AuthenticatedUser,
) -> Result<String, MyErrors> {
  let storage = StorageService::new()?;
  let signature_filename = user_bi
    .ok_or(UnexpectedError::ShouldNotHappen)?
    .signature_file_name
    .ok_or(UnexpectedError::ShouldNotHappen)?;

  Ok(storage.signature_url(&signature_filename))
}

#[debug_handler]
pub async fn upload_signature(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  mut multipart: Multipart,
) -> Result<status::StatusCode, MyErrors> {
  let field = multipart
    .next_field()
    .await
    .map_err(|_| ApplicationError::BadRequest)?
    .ok_or(ApplicationError::BadRequest)?;

  let field_name = field.name().ok_or(ApplicationError::BadRequest)?;
  if field_name != "signature" {
    return Err(ApplicationError::BadRequest.into());
  }

  let signature_data = field
    .bytes()
    .await
    .map_err(|_| ApplicationError::UnprocessableEntity)?;

  let img = image::load_from_memory(&signature_data).map_err(|e| {
    tracing::error!("Failed to load image: {}", e);
    ApplicationError::UnprocessableEntity
  })?;

  let resized = img.resize_exact(314, 156, FilterType::Lanczos3);

  let mut png_bytes: Vec<u8> = Vec::new();
  resized
    .write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)
    .map_err(|e| {
      tracing::error!("Failed to encode image: {}", e);
      ApplicationError::UnprocessableEntity
    })?;

  let filename = format!(
    "{}_{}_{}",
    &current_user.first_name.to_lowercase(),
    &current_user.last_name.to_lowercase(),
    &current_user.id.to_string()
  );

  let storage_service = services::storage::StorageService::new()?;
  storage_service
    .upload_signature(&png_bytes, &filename, "image/png")
    .await?;

  let mut business_information = current_user
    .find_related(UserBusinessInformations)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::UnprocessableEntity)?
    .into_active_model();

  business_information.signature_file_name = ActiveValue::Set(Some(filename));
  business_information.update(DB::get()).await?;

  Ok(status::StatusCode::NO_CONTENT)
}
