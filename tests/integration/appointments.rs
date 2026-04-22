use chrono::NaiveDate;
use opencab::models::{
  _entities::{medical_appointments, sea_orm_active_enums::PaymentMethod},
  medical_appointments::UpdateMedicalAppointmentParams,
};
use opencab::services::appointments::MedicalAppointmentExtractor;
use sea_orm::{EntityTrait, IntoActiveModel};
use serial_test::serial;

use crate::common::setup_db;
use crate::factories::{
  medical_appointment::AppointmentFactory, office::OfficeFactory, patient::PatientFactory,
  user::UserFactory,
};

struct Background {
  db: sea_orm::DatabaseConnection,
  user: opencab::models::_entities::users::Model,
  office: opencab::models::_entities::practitioner_offices::Model,
  patient: opencab::models::_entities::patients::Model,
}

async fn background() -> Background {
  let db = setup_db().await;
  let user = UserFactory::new().create(&db).await;
  let office = OfficeFactory::new()
    .name("Cabinet Central")
    .create_for_user(&db, user.id, 70)
    .await;
  let patient = PatientFactory::new()
    .first_name("Alice")
    .last_name("Dupont")
    .create(&db, user.id)
    .await;
  Background {
    db,
    user,
    office,
    patient,
  }
}

// ============================================================

mod create_an_appointment {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_it_is_saved_with_the_correct_date() {
    // Given
    let bg = background().await;

    // When
    let appointment = AppointmentFactory::new()
      .date("2026-03-15")
      .price(5000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;

    // Then
    assert_eq!(
      appointment.date,
      NaiveDate::parse_from_str("2026-03-15", "%Y-%m-%d").unwrap()
    );
  }
}

// ============================================================

mod create_an_appointment_with_a_payment_method {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_payment_method_is_saved() {
    // Given
    let bg = background().await;

    // When
    let appointment = AppointmentFactory::new()
      .date("2026-03-15")
      .price(5000)
      .payment_method(PaymentMethod::Cash)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;

    // Then
    assert_eq!(appointment.payment_method, Some(PaymentMethod::Cash));
  }
}

// ============================================================

mod update_an_appointment_date {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_new_date_is_saved() {
    // Given
    let bg = background().await;
    let appointment = AppointmentFactory::new()
      .date("2026-03-15")
      .price(5000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;
    let appointment_id = appointment.id;

    // When
    let new_date = NaiveDate::parse_from_str("2026-04-20", "%Y-%m-%d").unwrap();
    appointment
      .into_active_model()
      .update(
        &bg.db,
        &UpdateMedicalAppointmentParams {
          date: new_date,
          price_in_cents: 5000,
          practitioner_office_id: bg.office.id,
          payment_method: None,
        },
      )
      .await
      .unwrap();

    // Then
    let updated = medical_appointments::Entity::find_by_id(appointment_id)
      .one(&bg.db)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(updated.date, new_date);
  }
}

// ============================================================

mod extract_appointments_within_a_date_range {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_all_appointments_in_range_are_returned() {
    // Given
    let bg = background().await;
    AppointmentFactory::new()
      .date("2026-03-10")
      .price(3000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;
    AppointmentFactory::new()
      .date("2026-03-20")
      .price(4500)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;

    // When
    let start = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str("2026-03-31", "%Y-%m-%d").unwrap();
    let results = MedicalAppointmentExtractor::for_user(&bg.user)
      .extract(&bg.db, start, end)
      .await
      .unwrap();

    // Then
    assert_eq!(results.len(), 2);
  }
}

// ============================================================

mod extract_appointments_outside_the_date_range {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_no_appointments_are_returned() {
    // Given
    let bg = background().await;
    AppointmentFactory::new()
      .date("2026-02-15")
      .price(3000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;

    // When
    let start = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str("2026-03-31", "%Y-%m-%d").unwrap();
    let results = MedicalAppointmentExtractor::for_user(&bg.user)
      .extract(&bg.db, start, end)
      .await
      .unwrap();

    // Then
    assert_eq!(results.len(), 0);
  }
}

// ============================================================

mod extract_appointments_with_revenue_share {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_the_revenue_share_percentage_is_included() {
    // Given
    let bg = background().await;
    AppointmentFactory::new()
      .date("2026-03-10")
      .price(10000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;

    // When
    let start = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str("2026-03-31", "%Y-%m-%d").unwrap();
    let results = MedicalAppointmentExtractor::for_user(&bg.user)
      .extract(&bg.db, start, end)
      .await
      .unwrap();

    // Then
    assert_eq!(results[0].revenue_share_percentage, 70.0);
  }
}

// ============================================================

mod extract_appointments_with_multiple_offices {
  use super::*;

  #[tokio::test]
  #[serial]
  async fn then_each_office_has_its_own_revenue_share() {
    // Given
    let bg = background().await;
    let second_office = OfficeFactory::new()
      .name("Cabinet Sud")
      .create_for_user(&bg.db, bg.user.id, 50)
      .await;
    AppointmentFactory::new()
      .date("2026-03-10")
      .price(10000)
      .create(&bg.db, bg.user.id, bg.patient.id, bg.office.id)
      .await;
    AppointmentFactory::new()
      .date("2026-03-15")
      .price(8000)
      .create(&bg.db, bg.user.id, bg.patient.id, second_office.id)
      .await;

    // When
    let start = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str("2026-03-31", "%Y-%m-%d").unwrap();
    let results = MedicalAppointmentExtractor::for_user(&bg.user)
      .extract(&bg.db, start, end)
      .await
      .unwrap();

    // Then
    assert_eq!(results.len(), 2);
    let central = results
      .iter()
      .find(|r| r.office.name == "Cabinet Central")
      .unwrap();
    let sud = results
      .iter()
      .find(|r| r.office.name == "Cabinet Sud")
      .unwrap();
    assert_eq!(central.revenue_share_percentage, 70.0);
    assert_eq!(sud.revenue_share_percentage, 50.0);
  }
}
