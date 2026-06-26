use axum::{extract::Path, http::status, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

use crate::{
  middleware::context::Ctx,
  models::{
    _entities::{company_interventions, practitioner_companies},
    my_errors::{application_error::ApplicationError, MyErrors},
  },
};

pub async fn list_interventions(
  ctx: Ctx,
  Path(company_id): Path<i32>,
) -> Result<Json<Vec<company_interventions::Model>>, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx.authorize().user_owning_resource(&company).await.run_complete()?;

  let interventions = company_interventions::Entity::find()
    .filter(company_interventions::Column::CompanyId.eq(company_id))
    .order_by_desc(company_interventions::Column::IssueDate)
    .all(&ctx.db)
    .await?;

  Ok(Json(interventions))
}

pub async fn delete(ctx: Ctx, Path((company_id, intervention_id)): Path<(i32, i32)>) -> Result<status::StatusCode, MyErrors> {
  let company = practitioner_companies::Entity::find_by_id(company_id)
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  let intervention = company_interventions::Entity::find()
    .filter(company_interventions::Column::Id.eq(intervention_id))
    .filter(company_interventions::Column::CompanyId.eq(company.id))
    .one(&ctx.db)
    .await?
    .ok_or(ApplicationError::not_found())?;

  ctx
    .authorize()
    .user_owning_resource(&company)
    .await
    .user_owning_resource(&intervention)
    .await
    .run_complete()?;

  intervention.into_active_model().delete(&ctx.db).await?;

  Ok(status::StatusCode::NO_CONTENT)
}
