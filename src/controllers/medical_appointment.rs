use axum::{
  debug_handler,
  extract::{Path, State},
  http::status,
  Json,
};
use chrono::NaiveDate;
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter};
use serde::Deserialize;

use crate::{
  app_state::AppState,
  auth::statement::AuthStatement,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::{medical_appointments, sea_orm_active_enums::PaymentMethod},
    medical_appointments::{CreateMedicalAppointmentParams, UpdateMedicalAppointmentParams},
    my_errors::{application_error::ApplicationError, MyErrors},
  },
  services::{self, invoice::GenerateInvoiceParams},
};

#[derive(Debug, Deserialize)]
pub struct MedicalAppointmentPayload {
  date: String,
  practitioner_office_id: i32,
  price_in_cents: i32,
  payment_method: Option<PaymentMethod>,
}

pub async fn delete(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Path((patient_id, appointment_id)): Path<(i32, i32)>,
) -> Result<status::StatusCode, MyErrors> {
  let medical_appointment = medical_appointments::Entity::find_by_id(appointment_id)
    .filter(medical_appointments::Column::PatientId.eq(patient_id))
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&medical_appointment)
    .await
    .run_complete()?;

  medical_appointment.delete(&state.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn generate_invoice(
  State(state): State<AppState>,
  authorize: AuthStatement,
  AuthenticatedUser(current_user, user_business_info): AuthenticatedUser,
  Path((patient_id, appointment_id)): Path<(i32, i32)>,
) -> Result<status::StatusCode, MyErrors> {
  let medical_appointment = medical_appointments::Entity::find_by_id(appointment_id)
    .filter(medical_appointments::Column::PatientId.eq(patient_id))
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&medical_appointment)
    .await
    .run_complete()?;

  let Some(business_info) = user_business_info else {
    return Err(ApplicationError::UnprocessableEntity.into());
  };

  let invoice_generation_params = GenerateInvoiceParams {
    amount: medical_appointment.price_in_cents as f32 / 100.0,
    date: medical_appointment.date.format("%Y-%m-%d").to_string(),
    office_id: medical_appointment.practitioner_office(&state.db).await?.id,
  };

  let generated_invoice = services::invoice::generate_patient_invoice(
    &patient_id,
    &invoice_generation_params,
    &current_user,
  )
  .await?;

  if generated_invoice.patient_email.is_none() {
    return Err(ApplicationError::new("no_email_set_on_patient").into());
  }

  services::invoice::send_invoice(&state, &generated_invoice, &current_user, &business_info)
    .await?;

  Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn update(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Path((patient_id, appointment_id)): Path<(i32, i32)>,
  Json(params): Json<MedicalAppointmentPayload>,
) -> Result<status::StatusCode, MyErrors> {
  let medical_appointment = medical_appointments::Entity::find_by_id(appointment_id)
    .filter(medical_appointments::Column::PatientId.eq(patient_id))
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&medical_appointment)
    .await
    .run_complete()?;

  // Parse date string in YYYY-MM-DD format
  let appointment_date = NaiveDate::parse_from_str(&params.date, "%Y-%m-%d")?;

  let medical_appointments_params = UpdateMedicalAppointmentParams {
    date: appointment_date,
    practitioner_office_id: params.practitioner_office_id,
    price_in_cents: params.price_in_cents,
    payment_method: params.payment_method,
  };

  medical_appointment
    .into_active_model()
    .update(&state.db, &medical_appointments_params)
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn create(
  State(state): State<AppState>,
  authorize: AuthStatement,
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Path(patient_id): Path<i32>,
  Json(params): Json<MedicalAppointmentPayload>,
) -> Result<status::StatusCode, MyErrors> {
  authorize.authenticated_user().run_complete()?;

  // Parse date string in YYYY-MM-DD format
  let appointment_date = NaiveDate::parse_from_str(&params.date, "%Y-%m-%d")?;

  let medical_appointments_params = CreateMedicalAppointmentParams {
    date: appointment_date,
    practitioner_office_id: params.practitioner_office_id,
    price_in_cents: params.price_in_cents,
    user_id: current_user.id,
    patient_id,
    payment_method: params.payment_method,
  };

  medical_appointments::ActiveModel::create(&state.db, &medical_appointments_params).await?;

  Ok(status::StatusCode::NO_CONTENT)
}
