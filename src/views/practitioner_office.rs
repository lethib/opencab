use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize, Serializer};

use crate::models::_entities::{practitioner_offices, user_practitioner_offices};

fn serialize_decimal_as_f64<S: Serializer>(value: &Option<Decimal>, s: S) -> Result<S::Ok, S::Error> {
  match value {
    Some(dec) => s.serialize_f64(dec.to_string().parse::<f64>().map_err(serde::ser::Error::custom)?),
    None => s.serialize_none(),
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PractitionerOffice {
  id: i32,
  pub name: String,
  pub address_line_1: String,
  pub address_zip_code: String,
  pub address_city: String,
  #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_decimal_as_f64")]
  pub revenue_share_percentage: Option<Decimal>,
}

impl PractitionerOffice {
  pub fn new(office: &practitioner_offices::Model) -> Self {
    Self {
      id: office.id,
      name: office.name.clone(),
      address_line_1: office.address_line_1.clone(),
      address_zip_code: office.address_zip_code.clone(),
      address_city: office.address_city.clone(),
      revenue_share_percentage: None,
    }
  }

  pub fn new_with_upo(office: &practitioner_offices::Model, upo: &user_practitioner_offices::Model) -> Self {
    Self {
      revenue_share_percentage: Some(upo.revenue_share_percentage),
      ..Self::new(office)
    }
  }
}
