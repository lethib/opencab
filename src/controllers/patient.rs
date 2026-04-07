use axum::{
  debug_handler,
  extract::{Path, Query, State},
  http::status,
  Json,
};
use base64::Engine;
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchParams {
  pub q: String,
  pub page: Option<u64>,
}

use crate::{
  app_state::AppState,
  auth::statement::AuthStatement,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::{
      medical_appointments, patients, practitioner_offices, sea_orm_active_enums::PaymentMethod,
    },
    medical_appointments::{ActiveModel as MedicalAppointments, CreateMedicalAppointmentParams},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
    patients::CreatePatientParams,
  },
  services::{self, invoice::GenerateInvoiceParams},
  views::{medical_appointments::MedicalAppointmentResponse, patient::PatientResponse},
};

#[debug_handler]
pub async fn get(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Path(patient_id): Path<i32>,
) -> Result<Json<PatientResponse>, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  Ok(Json(PatientResponse::new(&patient)))
}

#[debug_handler]
pub async fn create(
  State(_state): State<AppState>,
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(create_patient_params): Json<CreatePatientParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  services::patients::create(&create_patient_params, &current_user).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

#[debug_handler]
pub async fn update(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Path(patient_id): Path<i32>,
  Json(patient_params): Json<CreatePatientParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  services::patients::update(&patient, &patient_params).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

#[debug_handler]
pub async fn delete(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Path(patient_id): Path<i32>,
) -> Result<status::StatusCode, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&state.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  patient.delete(&state.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn search(
  State(_state): State<AppState>,
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let page = params.page.unwrap_or(1);

  let query = if params.q.trim().is_empty() {
    ""
  } else {
    &params.q
  };

  let (patients, total_pages) =
    services::patients::search_paginated(query, page, &current_user).await?;

  let patient_responses: Vec<PatientResponse> =
    patients.iter().map(PatientResponse::from_model).collect();

  Ok(Json(serde_json::json!({
    "paginated_data": patient_responses,
    "pagination": {
      "page": page,
      "per_page": 10,
      "total_pages": total_pages,
      "has_more": page < total_pages
    }
  })))
}

#[derive(Debug, Deserialize)]
pub struct InvoiceGenerationParams {
  should_be_sent_by_email: bool,
  payment_method: Option<PaymentMethod>,
  invoice_params: GenerateInvoiceParams,
}

#[debug_handler]
pub async fn generate_invoice(
  State(state): State<AppState>,
  AuthenticatedUser(current_user, user_bi): AuthenticatedUser,
  Path(patient_id): Path<i32>,
  Json(params): Json<InvoiceGenerationParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  if params.invoice_params.amount <= 0.0 {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  if params.invoice_params.amount > (i32::MAX as f32 / 100.0) {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  let invoice_generated =
    services::invoice::generate_patient_invoice(&patient_id, &params.invoice_params, &current_user)
      .await?;

  let medical_appointment_params = CreateMedicalAppointmentParams {
    user_id: current_user.id,
    patient_id,
    practitioner_office_id: params.invoice_params.office_id,
    payment_method: params.payment_method.clone(),
    date: NaiveDate::parse_from_str(&params.invoice_params.date, "%Y-%m-%d")?,
    price_in_cents: (params.invoice_params.amount * 100.0).round() as i32,
  };

  MedicalAppointments::create(&state.db, &medical_appointment_params).await?;

  if params.should_be_sent_by_email {
    match &user_bi {
      Some(business_information) => {
        services::invoice::send_invoice(
          &state,
          &invoice_generated,
          &current_user,
          business_information,
        )
        .await?
      }
      None => return Err(ApplicationError::UnprocessableEntity.into()),
    }
  }

  Ok(Json(serde_json::json!({
    "pdf_data": base64::prelude::BASE64_STANDARD.encode(&invoice_generated.pdf_data),
    "filename": invoice_generated.filename
  })))
}

#[debug_handler]
pub async fn get_medical_appointments(
  State(state): State<AppState>,
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Path(patient_id): Path<i32>,
) -> Result<Json<Vec<MedicalAppointmentResponse>>, MyErrors> {
  let medical_appointments = medical_appointments::Entity::find()
    .filter(medical_appointments::Column::PatientId.eq(patient_id))
    .filter(medical_appointments::Column::UserId.eq(current_user.id))
    .order_by_desc(medical_appointments::Column::Date)
    .find_also_related(practitioner_offices::Entity)
    .all(&state.db)
    .await?
    .into_iter()
    .map(|appointment| {
      Ok(MedicalAppointmentResponse::new(
        &appointment.0,
        &appointment.1.ok_or(UnexpectedError::ShouldNotHappen)?,
      ))
    })
    .collect::<Result<Vec<_>, MyErrors>>()?;

  Ok(Json(medical_appointments))
}
