use chrono::NaiveDate;
use cucumber::{given, then, when};
use opencab::models::{
  _entities::{
    medical_appointments, practitioner_offices, sea_orm_active_enums::PaymentMethod,
    user_practitioner_offices,
  },
  medical_appointments::UpdateMedicalAppointmentParams,
  user_practitioner_offices::CreateLinkParams,
};
use opencab::services::appointments::MedicalAppointmentExtractor;
use sea_orm::{prelude::Decimal, EntityTrait, IntoActiveModel};

use crate::{
  factories::{
    medical_appointment::AppointmentFactory, office::OfficeFactory, patient::PatientFactory,
    user::UserFactory,
  },
  AppWorld,
};

#[given("a practitioner exists")]
async fn practitioner_exists(world: &mut AppWorld) {
  world.appointments.user = Some(UserFactory::new().create(&world.db).await);
}

async fn create_office_with_revenue_share(
  world: &mut AppWorld,
  name: &str,
  revenue_share: i64,
) -> practitioner_offices::Model {
  let user = world.appointments.user.as_ref().unwrap();
  let office = OfficeFactory::new().name(name).create(&world.db).await;
  user_practitioner_offices::ActiveModel::create(
    &world.db,
    &CreateLinkParams {
      user_id: user.id,
      practitioner_office_id: office.id,
      revenue_share_percentage: Decimal::from(revenue_share),
    },
  )
  .await
  .unwrap();
  office
}

#[given(expr = "a practitioner office {string} exists with revenue share {int}")]
async fn practitioner_office_exists(world: &mut AppWorld, name: String, revenue_share: i64) {
  let office = create_office_with_revenue_share(world, &name, revenue_share).await;
  world.appointments.office = Some(office);
}

#[given(expr = "a second office {string} exists with revenue share {int}")]
async fn second_office_exists(world: &mut AppWorld, name: String, revenue_share: i64) {
  let office = create_office_with_revenue_share(world, &name, revenue_share).await;
  world.appointments.second_office = Some(office);
}

#[given(expr = "a patient {string} {string} exists")]
async fn patient_exists(world: &mut AppWorld, first_name: String, last_name: String) {
  let user_id = world.appointments.user.as_ref().unwrap().id;
  world.appointments.patient = Some(
    PatientFactory::new()
      .first_name(&first_name)
      .last_name(&last_name)
      .create(&world.db, user_id)
      .await,
  );
}

async fn do_create_appointment(world: &mut AppWorld, date_str: &str, price: i32) {
  let user_id = world.appointments.user.as_ref().unwrap().id;
  let patient_id = world.appointments.patient.as_ref().unwrap().id;
  let office_id = world.appointments.office.as_ref().unwrap().id;
  world.appointments.appointment = Some(
    AppointmentFactory::new()
      .date(date_str)
      .price(price)
      .create(&world.db, user_id, patient_id, office_id)
      .await,
  );
}

#[given(expr = "an appointment on {string} at price {int}")]
async fn given_appointment(world: &mut AppWorld, date_str: String, price: i32) {
  do_create_appointment(world, &date_str, price).await;
}

#[given(expr = "an appointment on {string} at price {int} at office {string}")]
async fn given_appointment_at_office(
  world: &mut AppWorld,
  date_str: String,
  price: i32,
  office_name: String,
) {
  let user_id = world.appointments.user.as_ref().unwrap().id;
  let patient_id = world.appointments.patient.as_ref().unwrap().id;
  let office_id = world
    .appointments
    .all_offices()
    .find(|o| o.name == office_name)
    .unwrap_or_else(|| panic!("office '{}' not found", office_name))
    .id;
  AppointmentFactory::new()
    .date(&date_str)
    .price(price)
    .create(&world.db, user_id, patient_id, office_id)
    .await;
}

#[when(expr = "I create an appointment on {string} at price {int}")]
async fn when_create_appointment(world: &mut AppWorld, date_str: String, price: i32) {
  do_create_appointment(world, &date_str, price).await;
}

#[when(expr = "I create an appointment on {string} at price {int} with payment {string}")]
async fn create_appointment_with_payment(
  world: &mut AppWorld,
  date_str: String,
  price: i32,
  payment: String,
) {
  let user_id = world.appointments.user.as_ref().unwrap().id;
  let patient_id = world.appointments.patient.as_ref().unwrap().id;
  let office_id = world.appointments.office.as_ref().unwrap().id;
  world.appointments.appointment = Some(
    AppointmentFactory::new()
      .date(&date_str)
      .price(price)
      .payment_method(parse_payment_method(&payment))
      .create(&world.db, user_id, patient_id, office_id)
      .await,
  );
}

#[when(expr = "I update the appointment date to {string}")]
async fn update_appointment_date(world: &mut AppWorld, date_str: String) {
  let appointment = world.appointments.appointment.take().unwrap();
  let appointment_id = appointment.id;
  let office_id = world.appointments.office.as_ref().unwrap().id;
  let new_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
  let params = UpdateMedicalAppointmentParams {
    date: new_date,
    price_in_cents: appointment.price_in_cents,
    practitioner_office_id: office_id,
    payment_method: appointment.payment_method.clone(),
  };
  appointment
    .into_active_model()
    .update(&world.db, &params)
    .await
    .unwrap();

  let updated = medical_appointments::Entity::find_by_id(appointment_id)
    .one(&world.db)
    .await
    .unwrap()
    .unwrap();
  world.appointments.appointment = Some(updated);
}

#[when(expr = "I extract appointments between {string} and {string}")]
async fn extract_appointments(world: &mut AppWorld, start_str: String, end_str: String) {
  let user = world.appointments.user.as_ref().unwrap();
  let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").unwrap();
  let end = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").unwrap();
  let results = MedicalAppointmentExtractor::for_user(user)
    .extract(&world.db, start, end)
    .await
    .unwrap();
  world.appointments.extracted = results;
}

#[then(expr = "the appointment is saved with date {string}")]
fn appointment_saved(world: &mut AppWorld, date_str: String) {
  let appointment = world.appointments.appointment.as_ref().unwrap();
  let expected = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
  assert_eq!(appointment.date, expected);
}

#[then(expr = "the appointment payment method is {string}")]
fn appointment_payment_method(world: &mut AppWorld, payment: String) {
  let appointment = world.appointments.appointment.as_ref().unwrap();
  assert_eq!(
    appointment.payment_method,
    Some(parse_payment_method(&payment))
  );
}

#[then(expr = "the appointment date is {string}")]
fn appointment_date(world: &mut AppWorld, date_str: String) {
  let appointment = world.appointments.appointment.as_ref().unwrap();
  let expected = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
  assert_eq!(appointment.date, expected);
}

#[then(expr = "{int} appointments are returned")]
fn appointments_count(world: &mut AppWorld, count: usize) {
  assert_eq!(world.appointments.extracted.len(), count);
}

#[then(expr = "the first extracted appointment has a revenue share of {float}")]
fn first_appointment_revenue_share(world: &mut AppWorld, expected: f64) {
  let appointment_details = &world.appointments.extracted[0];
  assert_eq!(appointment_details.revenue_share_percentage, expected);
}

#[then(expr = "the extracted appointment for office {string} has a revenue share of {float}")]
fn appointment_revenue_share_for_office(world: &mut AppWorld, office_name: String, expected: f64) {
  let appointment_details = world
    .appointments
    .extracted
    .iter()
    .find(|a_d| a_d.office.name == office_name)
    .unwrap_or_else(|| panic!("no extracted appointment for office '{}'", office_name));
  assert_eq!(appointment_details.office.name, office_name);
  assert_eq!(appointment_details.revenue_share_percentage, expected);
}

fn parse_payment_method(s: &str) -> PaymentMethod {
  match s {
    "cash" => PaymentMethod::Cash,
    "card" => PaymentMethod::Card,
    "check" => PaymentMethod::Check,
    "transfer" => PaymentMethod::Transfer,
    _ => panic!("unknown payment method: {}", s),
  }
}
