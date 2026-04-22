use axum::http::StatusCode;
use opencab::models::{
  _entities::{practitioner_offices, user_practitioner_offices},
  practitioner_offices::PractitionerOfficeParams,
};
use sea_orm::ActiveValue::Set;
use sea_orm::{
  prelude::Decimal, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serial_test::serial;
use std::str::FromStr;

use crate::common::setup_db;
use crate::factories::{office::OfficeFactory, user::UserFactory};

// ============================================================

mod create_an_office_linked_to_a_practitioner {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_it_is_linked_with_the_correct_revenue_share() {
    // Given
    let db = setup_db().await;
    let user = UserFactory::new().create(&db).await;

    // When
    let office = OfficeFactory::new()
      .name("Cabinet du Sud")
      .create_for_user(&db, user.id, 30)
      .await;

    // Then
    let link = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::PractitionerOfficeId.eq(office.id))
      .filter(user_practitioner_offices::Column::UserId.eq(user.id))
      .one(&db)
      .await
      .unwrap()
      .expect("link should exist");
    assert_eq!(
      link.revenue_share_percentage,
      Decimal::from_str("30").unwrap()
    );
  }
}

// ============================================================

mod create_an_office_with_an_invalid_zip_code {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_creation_fails_with_unprocessable_entity() {
    // Given
    let db = setup_db().await;

    // When
    let result = practitioner_offices::ActiveModel::create(
      &db,
      &PractitionerOfficeParams {
        name: "Cabinet Test".to_string(),
        address_line_1: "1 rue de la Paix".to_string(),
        address_zip_code: "INVALID".to_string(),
        address_city: "Paris".to_string(),
      },
    )
    .await;

    // Then
    let err = result.expect_err("creation should have failed");
    assert_eq!(err.code, StatusCode::UNPROCESSABLE_ENTITY);
  }
}

// ============================================================

mod update_an_office_revenue_share {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_new_revenue_share_is_saved() {
    // Given
    let db = setup_db().await;
    let user = UserFactory::new().create(&db).await;
    let office = OfficeFactory::new()
      .name("Cabinet du Nord")
      .create_for_user(&db, user.id, 20)
      .await;

    // When
    let mut link = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::PractitionerOfficeId.eq(office.id))
      .filter(user_practitioner_offices::Column::UserId.eq(user.id))
      .one(&db)
      .await
      .unwrap()
      .unwrap()
      .into_active_model();
    link.revenue_share_percentage = Set(Decimal::from_str("45").unwrap());
    link.update(&db).await.unwrap();

    // Then
    let updated_link = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::PractitionerOfficeId.eq(office.id))
      .filter(user_practitioner_offices::Column::UserId.eq(user.id))
      .one(&db)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(
      updated_link.revenue_share_percentage,
      Decimal::from_str("45").unwrap()
    );
  }
}

// ============================================================

mod update_an_office_name {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_new_name_is_saved() {
    // Given
    let db = setup_db().await;
    let user = UserFactory::new().create(&db).await;
    let office = OfficeFactory::new()
      .name("Old Name")
      .create_for_user(&db, user.id, 25)
      .await;

    // When
    let mut active = office.into_active_model();
    active.name = Set("New Name".to_string());
    let updated = active.update(&db).await.unwrap();

    // Then
    assert_eq!(updated.name, "New Name");
  }
}

// ============================================================

mod update_an_office_name_with_extra_whitespace {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_name_is_trimmed() {
    // Given
    let db = setup_db().await;
    let user = UserFactory::new().create(&db).await;
    let office = OfficeFactory::new()
      .name("My Office")
      .create_for_user(&db, user.id, 10)
      .await;

    // When
    let mut active = office.into_active_model();
    active.name = Set("  Trimmed Name  ".trim().to_string());
    let updated = active.update(&db).await.unwrap();

    // Then
    assert_eq!(updated.name, "Trimmed Name");
  }
}
