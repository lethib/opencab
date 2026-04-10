use axum::{debug_handler, extract::Path, Json};
use sea_orm::{prelude::Decimal, EntityTrait, IntoActiveModel, ModelTrait};
use serde::Deserialize;

use crate::{
  auth::statement::AuthStatement,
  db::DB,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::practitioner_offices,
    my_errors::{application_error::ApplicationError, MyErrors},
    practitioner_offices::PractitionerOfficeParams,
  },
  services,
};

#[derive(Deserialize)]
pub struct OfficeParams {
  pub office: PractitionerOfficeParams,
  pub revenue_share_percentage: Decimal,
}

#[debug_handler]
pub async fn create(
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Json(params): Json<OfficeParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  if params.revenue_share_percentage < Decimal::ZERO
    || params.revenue_share_percentage > Decimal::ONE_HUNDRED
  {
    return Err(ApplicationError::BadRequest.into());
  }

  services::practitioner_office::create(
    &params.office,
    &current_user,
    params.revenue_share_percentage,
  )
  .await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

#[debug_handler]
pub async fn update(
  authorize: AuthStatement,
  AuthenticatedUser(current_user, _): AuthenticatedUser,
  Path(office_id): Path<i32>,
  Json(params): Json<OfficeParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let office = practitioner_offices::Entity::find_by_id(office_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&office)
    .await
    .run_complete()?;

  if params.revenue_share_percentage < Decimal::ZERO
    || params.revenue_share_percentage > Decimal::ONE_HUNDRED
  {
    return Err(ApplicationError::BadRequest.into());
  }

  services::practitioner_office::update(
    office.into_active_model(),
    &params.office,
    &current_user,
    params.revenue_share_percentage,
  )
  .await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

#[debug_handler]
pub async fn destroy(
  authorize: AuthStatement,
  Path(office_id): Path<i32>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let office = practitioner_offices::Entity::find_by_id(office_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  authorize
    .user_owning_resource(&office)
    .await
    .run_complete()?;

  office.clone().delete(DB::get()).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}
