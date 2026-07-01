use axum::{extract::Path, http::status, Json};
use base64::Engine;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::Deserialize;
use std::str::FromStr;

use crate::{
  middleware::context::Ctx,
  models::{
    _entities::{company_interventions, practitioner_companies, practitioner_offices, prelude},
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

pub async fn index(ctx: Ctx) -> Result<Json<Vec<practitioner_companies::Model>>, MyErrors> {
  let companies = practitioner_companies::Entity::find()
    .filter(practitioner_companies::COLUMN.user_id.eq(ctx.current_user.id))
    .all(&ctx.db)
    .await?;

  Ok(Json(companies))
}

pub async fn get(ctx: Ctx, Path(company_id): Path<i32>) -> Result<Json<practitioner_companies::Model>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx.authorize().user_owning_resource(&company).await.run_complete()?;

  Ok(Json(company))
}

pub async fn create(ctx: Ctx, Json(params): Json<CompanyParams>) -> Result<status::StatusCode, MyErrors> {
  practitioner_companies::ActiveModel::create(&ctx.db, ctx.current_user.id, &params).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn update(
  ctx: Ctx,
  Path(company_id): Path<i32>,
  Json(params): Json<CompanyParams>,
) -> Result<status::StatusCode, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx.authorize().user_owning_resource(&company).await.run_complete()?;

  company.into_active_model().update_from_params(&ctx.db, &params).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn delete(ctx: Ctx, Path(company_id): Path<i32>) -> Result<status::StatusCode, MyErrors> {
  let company = prelude::PractitionerCompanies::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx.authorize().user_owning_resource(&company).await.run_complete()?;

  company.into_active_model().delete(&ctx.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn generate_invoice(
  ctx: Ctx,
  Path(company_id): Path<i32>,
  Json(params): Json<GenerateCompanyInvoiceParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx.authorize().user_owning_resource(&company).await.run_complete()?;

  let issue_date = chrono::NaiveDate::parse_from_str(&params.invoice_date, "%Y-%m-%d")?;
  let vat_rate = Decimal::from_str(&params.vat_rate).map_err(ApplicationError::unprocessable_entity)?;

  let intervention_params = InterventionParams {
    quantity: params.quantity,
    unit_price: params.unit_price_ht,
    vat_rate,
    issue_date,
    object: params.description,
  };

  let intervention =
    company_interventions::ActiveModel::create(&ctx.db, ctx.current_user.id, company_id, &intervention_params).await?;

  let practitioner_office = practitioner_offices::Entity::find_by_id(params.practitioner_office_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  let invoice =
    services::invoice::company_invoice::generate(&intervention, &ctx.current_user, practitioner_office, &ctx.db).await?;

  Ok(Json(serde_json::json!({
    "pdf_data": base64::prelude::BASE64_STANDARD.encode(invoice.data),
    "filename": invoice.filename,
  })))
}
