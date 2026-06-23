use axum::{
  extract::{Path, Query},
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
  middleware::context::Ctx,
  models::{
    _entities::{
      medical_appointments, patients, practitioner_offices, sea_orm_active_enums::PaymentMethod,
    },
    medical_appointments::{ActiveModel as MedicalAppointments, CreateMedicalAppointmentParams},
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
    patients::CreatePatientParams,
  },
  services::{self, invoice::patient_invoice::GenerateInvoiceParams},
  views::{medical_appointments::MedicalAppointmentResponse, patient::PatientResponse},
};

pub async fn get(ctx: Ctx, Path(patient_id): Path<i32>) -> Result<Json<PatientResponse>, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  Ok(Json(PatientResponse::new(&patient)))
}

pub async fn create(
  ctx: Ctx,
  Json(create_patient_params): Json<CreatePatientParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  services::patients::create(&create_patient_params, &ctx.current_user, &ctx.db).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn update(
  ctx: Ctx,
  Path(patient_id): Path<i32>,
  Json(patient_params): Json<CreatePatientParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  services::patients::update(&patient, &patient_params, &ctx.db).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn delete(ctx: Ctx, Path(patient_id): Path<i32>) -> Result<status::StatusCode, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  patient.delete(&ctx.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn search(
  ctx: Ctx,
  Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let page = params.page.unwrap_or(1);

  let query = if params.q.trim().is_empty() {
    ""
  } else {
    &params.q
  };

  let (patients, total_pages) =
    services::patients::search_paginated(query, page, &ctx.current_user, &ctx.db).await?;

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

pub async fn generate_invoice(
  ctx: Ctx,
  Path(patient_id): Path<i32>,
  Json(params): Json<InvoiceGenerationParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let patient = patients::Entity::find_by_id(patient_id)
    .filter(patients::Column::UserId.eq(ctx.current_user.id))
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .user_owning_resource(&patient)
    .await
    .run_complete()?;

  if params.invoice_params.amount <= 0.0 {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  if params.invoice_params.amount > (i32::MAX as f32 / 100.0) {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  let generated_invoice = services::invoice::patient_invoice::generate(
    &patient,
    &params.invoice_params,
    &ctx.current_user,
    false,
    &ctx.db,
  )
  .await?;

  let user_bi = ctx.current_user.business_information(&ctx.db).await?;

  if params.should_be_sent_by_email {
    if let Some(email) = patient.email {
      generated_invoice
        .send_to(&email, &ctx.current_user, &user_bi.profession)
        .await?;
    } else {
      return Err(ApplicationError::UnprocessableEntity.into());
    }
  }

  let medical_appointment_params = CreateMedicalAppointmentParams {
    user_id: ctx.current_user.id,
    patient_id,
    practitioner_office_id: params.invoice_params.office_id,
    payment_method: params.payment_method.clone(),
    date: NaiveDate::parse_from_str(&params.invoice_params.date, "%Y-%m-%d")?,
    price_in_cents: (params.invoice_params.amount * 100.0).round() as i32,
  };

  MedicalAppointments::create(&ctx.db, &medical_appointment_params).await?;

  Ok(Json(serde_json::json!({
    "pdf_data": base64::prelude::BASE64_STANDARD.encode(&generated_invoice.data),
    "filename": generated_invoice.filename
  })))
}

pub async fn get_medical_appointments(
  ctx: Ctx,
  Path(patient_id): Path<i32>,
) -> Result<Json<Vec<MedicalAppointmentResponse>>, MyErrors> {
  let medical_appointments = medical_appointments::Entity::find()
    .filter(medical_appointments::Column::PatientId.eq(patient_id))
    .filter(medical_appointments::Column::UserId.eq(ctx.current_user.id))
    .order_by_desc(medical_appointments::Column::Date)
    .find_also_related(practitioner_offices::Entity)
    .all(&ctx.db)
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
