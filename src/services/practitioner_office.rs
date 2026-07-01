use sea_orm::{
  prelude::Decimal, ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
  TransactionTrait,
};

use crate::models::{
  _entities::{practitioner_offices, user_practitioner_offices},
  my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  practitioner_offices::PractitionerOfficeParams,
  user_practitioner_offices::CreateLinkParams,
  users::users,
};

pub async fn update(
  mut office: practitioner_offices::ActiveModel,
  params: &PractitionerOfficeParams,
  linked_practitioner: &users::Model,
  revenue_share_percentage: Decimal,
  db: &DatabaseConnection,
) -> Result<(), MyErrors> {
  let office_id = office.id.clone().take().ok_or(UnexpectedError::should_not_happen())?;

  let mut user_practitioner_office = user_practitioner_offices::Entity::find()
    .filter(user_practitioner_offices::COLUMN.practitioner_office_id.eq(office_id))
    .filter(user_practitioner_offices::COLUMN.user_id.eq(linked_practitioner.id))
    .one(db)
    .await?
    .ok_or(ApplicationError::not_found())?
    .into_active_model();

  office.name = Set(params.name.trim().to_string());
  office.address_line_1 = Set(params.address_line_1.trim().to_string());
  office.address_zip_code = Set(params.address_zip_code.trim().to_string());
  office.address_city = Set(params.address_city.trim().to_string());

  user_practitioner_office.revenue_share_percentage = Set(revenue_share_percentage);

  let db_transaction = db.begin().await?;

  office.update(&db_transaction).await?;
  user_practitioner_office.update(&db_transaction).await?;

  db_transaction.commit().await?;

  Ok(())
}

pub async fn create(
  params: &PractitionerOfficeParams,
  linked_practitioner: &users::Model,
  revenue_share_percentage: Decimal,
  db: &DatabaseConnection,
) -> Result<(), MyErrors> {
  let db_transaction = db.begin().await?;

  let office = practitioner_offices::ActiveModel::create(&db_transaction, params).await?;

  user_practitioner_offices::ActiveModel::create(
    &db_transaction,
    &CreateLinkParams {
      user_id: linked_practitioner.id,
      revenue_share_percentage,
      practitioner_office_id: office.id,
    },
  )
  .await?;

  db_transaction.commit().await?;

  Ok(())
}
