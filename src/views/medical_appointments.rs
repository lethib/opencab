use serde::{Deserialize, Serialize};

use crate::{
  models::_entities::{medical_appointments, practitioner_offices, sea_orm_active_enums::PaymentMethod},
  views::practitioner_office::PractitionerOffice,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct MedicalAppointmentResponse {
  id: i32,
  date: String,
  price_in_cents: i32,
  payment_method: Option<PaymentMethod>,
  office: PractitionerOffice,
}

impl MedicalAppointmentResponse {
  pub fn new(medical_appointment: &medical_appointments::Model, office: &practitioner_offices::Model) -> Self {
    Self {
      id: medical_appointment.id,
      date: medical_appointment.date.format("%Y-%m-%d").to_string(),
      price_in_cents: medical_appointment.price_in_cents,
      payment_method: medical_appointment.payment_method.clone(),
      office: PractitionerOffice::new(office),
    }
  }
}
