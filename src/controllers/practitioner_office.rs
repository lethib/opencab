use axum::{extract::Path, Json};
use sea_orm::{prelude::Decimal, EntityTrait, IntoActiveModel, ModelTrait};
use serde::Deserialize;

use crate::{
  middleware::context::Ctx,
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

fn validate_revenue_share(revenue_share: &Decimal) -> Result<(), MyErrors> {
  if revenue_share < &Decimal::ZERO || revenue_share > &Decimal::ONE_HUNDRED {
    return Err(ApplicationError::bad_request("revenue_share_outside_window").into());
  }

  Ok(())
}

pub async fn create(
  ctx: Ctx,
  Json(params): Json<OfficeParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  validate_revenue_share(&params.revenue_share_percentage)?;

  services::practitioner_office::create(
    &params.office,
    &ctx.current_user,
    params.revenue_share_percentage,
    &ctx.db,
  )
  .await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn update(
  ctx: Ctx,
  Path(office_id): Path<i32>,
  Json(params): Json<OfficeParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let office = practitioner_offices::Entity::find_by_id(office_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx
    .authorize()
    .user_owning_resource(&office)
    .await
    .run_complete()?;

  validate_revenue_share(&params.revenue_share_percentage)?;

  services::practitioner_office::update(
    office.into_active_model(),
    &params.office,
    &ctx.current_user,
    params.revenue_share_percentage,
    &ctx.db,
  )
  .await?;

  Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn destroy(
  ctx: Ctx,
  Path(office_id): Path<i32>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  let office = practitioner_offices::Entity::find_by_id(office_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx
    .authorize()
    .user_owning_resource(&office)
    .await
    .run_complete()?;

  office.clone().delete(&ctx.db).await?;

  Ok(Json(serde_json::json!({ "success": true })))
}
