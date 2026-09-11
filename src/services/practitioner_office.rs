use sea_orm::{
  prelude::Decimal, sea_query::Query, ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
  ModelTrait, QueryFilter, QuerySelect, TransactionTrait,
};

use crate::models::{
  _entities::{medical_appointments, patients, practitioner_offices, user_practitioner_offices},
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

pub async fn delete(
  office: practitioner_offices::Model,
  also_delete_patients: bool,
  db: &DatabaseConnection,
) -> Result<(), MyErrors> {
  if !also_delete_patients {
    office.delete(db).await?;
    return Ok(());
  }

  let patient_ids_in_another_office = Query::select()
    .column(medical_appointments::COLUMN.patient_id)
    .from(medical_appointments::Entity)
    .and_where(medical_appointments::COLUMN.practitioner_office_id.ne(office.id))
    .to_owned();

  let patient_ids_to_delete = office
    .patients()
    .select_only()
    .column(patients::Column::Id)
    .filter(
      medical_appointments::COLUMN
        .patient_id
        .not_in_subquery(patient_ids_in_another_office),
    )
    .distinct()
    .into_tuple::<i32>()
    .all(db)
    .await?;

  let txn = db.begin().await?;

  patients::Entity::delete_many()
    .filter(patients::COLUMN.id.is_in(patient_ids_to_delete))
    .exec(&txn)
    .await?;

  office.delete(&txn).await?;

  txn.commit().await?;

  Ok(())
}
