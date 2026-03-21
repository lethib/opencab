use crate::initializers::get_services;
use crate::models::_entities::patients;
use crate::models::{
  my_errors::MyErrors,
  patients::{CreatePatientParams, Model as PatientModel},
  users,
};
use sea_orm::{ColumnTrait, Condition, QueryFilter};
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};

pub async fn create(
  patient_params: &CreatePatientParams,
  linked_to_user: &users::Model,
) -> Result<PatientModel, MyErrors> {
  let services = get_services();

  let created_patient =
    patients::ActiveModel::create(&services.db, patient_params, linked_to_user.id).await?;

  Ok(created_patient)
}

pub async fn update(
  patient: &patients::Model,
  patient_params: &CreatePatientParams,
) -> Result<(), MyErrors> {
  let services = get_services();

  patients::ActiveModel::update(&services.db, patient.id, patient_params).await?;

  Ok(())
}

pub async fn search_paginated(
  query: &str,
  page: u64,
  user: &users::Model,
) -> Result<(Vec<PatientModel>, u64), MyErrors> {
  let db = &get_services().db;

  // Build search condition for first_name and last_name (case-insensitive)
  let search_condition = Condition::any()
    .add(sea_orm::sea_query::Expr::cust_with_values(
      "LOWER(first_name) LIKE LOWER($1)",
      [format!("%{}%", query)],
    ))
    .add(sea_orm::sea_query::Expr::cust_with_values(
      "LOWER(last_name) LIKE LOWER($1)",
      [format!("%{}%", query)],
    ));

  // Query patients that belong to the current user and match the search
  let paginator = patients::Entity::find()
    .filter(patients::Column::UserId.eq(user.id))
    .filter(search_condition)
    .order_by_desc(patients::Column::UpdatedAt)
    .paginate(db, 10);

  let total_pages = paginator.num_pages().await?;
  let patients_with_optional_offices = paginator.fetch_page(page - 1).await?; // SeaORM uses 0-based pagination

  Ok((patients_with_optional_offices, total_pages))
}
