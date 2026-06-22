use axum::{extract::Path, http::status, Json};
use base64::Engine;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};
use serde::Deserialize;
use std::str::FromStr;

use crate::{
  middleware::context::Ctx,
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
  pub should_be_sent_by_email: bool,
}

pub async fn index(ctx: Ctx) -> Result<Json<Vec<practitioner_companies::Model>>, MyErrors> {
  let companies = practitioner_companies::Entity::find()
    .filter(practitioner_companies::Column::UserId.eq(ctx.current_user.id))
    .all(&ctx.db)
    .await?;

  Ok(Json(companies))
}

pub async fn get(
  ctx: Ctx,
  Path(company_id): Path<i32>,
) -> Result<Json<practitioner_companies::Model>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  Ok(Json(company))
}

pub async fn create(
  ctx: Ctx,
  Json(params): Json<CompanyParams>,
) -> Result<status::StatusCode, MyErrors> {
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
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  company.into_active_model().update(&ctx.db, &params).await?;

  Ok(status::StatusCode::NO_CONTENT)
}

pub async fn list_interventions(
  ctx: Ctx,
  Path(company_id): Path<i32>,
) -> Result<Json<Vec<company_interventions::Model>>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
    .authenticated_user()
    .user_owning_resource(&company)
    .await
    .run_complete()?;

  let interventions = company_interventions::Entity::find()
    .filter(company_interventions::Column::CompanyId.eq(company_id))
    .order_by_desc(company_interventions::Column::IssueDate)
    .all(&ctx.db)
    .await?;

  Ok(Json(interventions))
}

pub async fn generate_invoice(
  ctx: Ctx,
  Path(company_id): Path<i32>,
  Json(params): Json<GenerateCompanyInvoiceParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  ctx
    .authorize()
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
    &ctx.db,
    ctx.current_user.id,
    company_id,
    &intervention_params,
  )
  .await?;

  let practitioner_office = practitioner_offices::Entity::find_by_id(params.practitioner_office_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let invoice = services::invoice::company_invoice::generate(
    &intervention,
    &ctx.current_user,
    practitioner_office,
    &ctx.db,
  )
  .await?;

  Ok(Json(serde_json::json!({
    "pdf_data": base64::prelude::BASE64_STANDARD.encode(invoice.data),
    "filename": invoice.filename,
  })))
}
