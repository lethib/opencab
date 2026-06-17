use axum::{debug_handler, extract::Path, http::status, Json};
use base64::Engine;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};
use serde::Deserialize;
use std::str::FromStr;

use crate::{
  auth::statement::AuthStatement,
  db::DB,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::{company_interventions, practitioner_companies, practitioner_offices},
    company_interventions::InterventionParams,
    my_errors::{application_error::ApplicationError, MyErrors},
    practitioner_companies::CompanyParams,
  },
  services,
};

#[derive(Deserialize)]
pub struct GenerateCompanyInvoiceParams {
  pub invoice_date: String,
  pub description: String,
  pub quantity: i32,
  pub unit_price_ht: f32,
  pub vat_rate: String,
  pub practitioner_office_id: i32,
}

#[debug_handler]
pub async fn index(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
) -> Result<Json<Vec<practitioner_companies::Model>>, MyErrors> {
  let companies = practitioner_companies::Entity::find()
    .filter(practitioner_companies::Column::UserId.eq(current_user.id))
    .all(DB::get())
    .await?;

  Ok(Json(companies))
}

#[debug_handler]
pub async fn get(
  authorize: AuthStatement,
  Path(company_id): Path<i32>,
) -> Result<Json<practitioner_companies::Model>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  Ok(Json(company))
}

#[debug_handler]
pub async fn create(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(params): Json<CompanyParams>,
) -> Result<status::StatusCode, MyErrors> {
  practitioner_companies::ActiveModel::create(DB::get(), current_user.id, &params).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn update(
  authorize: AuthStatement,
  Path(company_id): Path<i32>,
  Json(params): Json<CompanyParams>,
) -> Result<status::StatusCode, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  company
    .into_active_model()
    .update(DB::get(), &params)
    .await?;

  Ok(status::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn list_interventions(
  authorize: AuthStatement,
  Path(company_id): Path<i32>,
) -> Result<Json<Vec<company_interventions::Model>>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  let interventions = company_interventions::Entity::find()
    .filter(company_interventions::Column::CompanyId.eq(company_id))
    .order_by_desc(company_interventions::Column::IssueDate)
    .all(DB::get())
    .await?;

  Ok(Json(interventions))
}

#[debug_handler]
pub async fn generate_invoice(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  authorize: AuthStatement,
  Path(company_id): Path<i32>,
  Json(params): Json<GenerateCompanyInvoiceParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  let issue_date = chrono::NaiveDate::parse_from_str(&params.invoice_date, "%Y-%m-%d")?;
  let vat_rate =
    Decimal::from_str(&params.vat_rate).map_err(|_| ApplicationError::UnprocessableEntity)?;

  let intervention_params = InterventionParams {
    quantity: params.quantity,
    unit_price: params.unit_price_ht,
    vat_rate,
    issue_date,
    object: params.description,
  };

  let intervention = company_interventions::ActiveModel::create(
    DB::get(),
    current_user.id,
    company_id,
    &intervention_params,
  )
  .await?;

  let practitioner_office = practitioner_offices::Entity::find_by_id(params.practitioner_office_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let pdf_data =
    services::invoice::company_invoice::generate(&intervention, &current_user, practitioner_office)
      .await?;

  let filename = format!(
    "{} Facture {} {}.pdf",
    current_user.full_name(),
    company.name,
    intervention.issue_date.format("%d_%m_%Y"),
  );

  Ok(Json(serde_json::json!({
    "pdf_data": base64::prelude::BASE64_STANDARD.encode(&pdf_data),
    "filename": filename,
  })))
}
