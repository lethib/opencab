use axum::{debug_handler, extract::Path, http::status, Json};
use sea_orm::{EntityTrait, IntoActiveModel};

use crate::{
  auth::statement::AuthStatement,
  db::DB,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::practitioner_companies,
    my_errors::{application_error::ApplicationError, MyErrors},
    practitioner_companies::CompanyParams,
  },
};

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
