use crate::models::patients;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct PatientResponse {
  id: i32,
  pid: Uuid,
  pub first_name: String,
  pub last_name: String,
  pub email: Option<String>,
  pub ssn: String,
  pub address_line_1: String,
  pub address_zip_code: String,
  pub address_city: String,
  pub address_country: String,
}

impl PatientResponse {
  #[must_use]
  pub fn new(patient: &patients::Model) -> Self {
    Self {
      id: patient.id,
      pid: patient.pid,
      first_name: patient.first_name.clone(),
      last_name: patient.last_name.clone(),
      email: patient.email.clone(),
      ssn: patient
        .decrypt_ssn()
        .unwrap_or_else(|_| "Unable to decrypt".to_string()),
      address_line_1: patient.address_line_1.clone(),
      address_zip_code: patient.address_zip_code.clone(),
      address_city: patient.address_city.clone(),
      address_country: patient.address_country.clone(),
    }
  }

  #[must_use]
  pub fn from_model(patient: &patients::Model) -> Self {
    Self {
      id: patient.id,
      pid: patient.pid,
      first_name: patient.first_name.clone(),
      last_name: patient.last_name.clone(),
      email: patient.email.clone(),
      ssn: patient
        .decrypt_ssn()
        .unwrap_or_else(|_| "Unable to decrypt".to_string()),
      address_line_1: patient.address_line_1.clone(),
      address_zip_code: patient.address_zip_code.clone(),
      address_city: patient.address_city.clone(),
      address_country: patient.address_country.clone(),
    }
  }
}
