use crate::models::_entities::user_business_informations;
use sea_orm::ActiveEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BusinessInformation {
  pub rpps_number: String,
  pub siret_number: String,
  pub adeli_number: Option<String>,
  pub signature_filename: Option<String>,
  pub profession: String,
  pub beneficiary_name: Option<String>,
  pub iban: Option<String>,
  pub bic: Option<String>,
}

impl BusinessInformation {
  #[must_use]
  pub fn new(business_information: &user_business_informations::Model) -> Self {
    Self {
      rpps_number: business_information.rpps_number.clone(),
      siret_number: business_information.siret_number.clone(),
      adeli_number: business_information.adeli_number.as_ref().filter(|s| !s.is_empty()).cloned(),
      signature_filename: business_information.signature_file_name.clone(),
      profession: business_information.profession.to_value().value.into(),
      beneficiary_name: business_information.beneficiary_name.clone(),
      iban: business_information.iban.clone(),
      bic: business_information.bic.clone(),
    }
  }
}
