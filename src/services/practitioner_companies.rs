use sea_orm::{IntoActiveModel, TransactionTrait, TryIntoModel};
use serde::Deserialize;
use validator::Validate;

use crate::{
  models::{
    _entities::practitioner_companies::{self, Model as PractitionerCompany, *},
    my_errors::{application_error::ApplicationError, MyErrors},
  },
  validators::address::is_address_valid,
};

pub struct PractitionerCompaniesService {
  pub company: PractitionerCompany,
}

impl PractitionerCompaniesService {
  pub fn for_company(company: PractitionerCompany) -> Self {
    Self { company }
  }

  pub fn create(params: CompanyParams) -> Result<ActiveModelEx, MyErrors> {
    let mut is_address_provided = false;
    params.validate()?;
    if let (Some(address_line_1), Some(zip_code)) = (&params.address_line_1, &params.address_zip_code) {
      is_address_provided = true;
      if !is_address_valid(address_line_1, zip_code) {
        return Err(ApplicationError::unprocessable_entity("invalid_address").into());
      }
    }

    Ok(
      practitioner_companies::ActiveModel::builder()
        .set_name(params.name.trim())
        .set_contact_name(params.contact_name.trim())
        .set_contact_email(params.contact_email.trim())
        .set_siret(params.siret.map(|siret| siret.trim().to_string()))
        .set_address_line_1(params.address_line_1.map(|al1| al1.trim().to_string()))
        .set_address_zip_code(params.address_zip_code.map(|zip_code| zip_code.trim().to_string()))
        .set_address_city(params.address_city.map(|city| city.trim().to_string()))
        .set_address_country(is_address_provided.then(|| "FRANCE".to_string())),
    )
  }

  pub fn update(self, params: CompanyParams) -> Result<ActiveModelEx, MyErrors> {
    let mut is_address_provided = false;
    params.validate()?;
    if let (Some(address_line_1), Some(zip_code)) = (&params.address_line_1, &params.address_zip_code) {
      is_address_provided = true;
      if !is_address_valid(address_line_1, zip_code) {
        return Err(ApplicationError::unprocessable_entity("invalid_address").into());
      }
    }

    let company = self.company.into_active_model().into_ex();

    Ok(
      company
        .set_name(params.name.trim())
        .set_contact_name(params.contact_name.trim())
        .set_contact_email(params.contact_email.trim())
        .set_siret(params.siret.map(|siret| siret.trim().to_string()))
        .set_address_line_1(params.address_line_1.map(|al1| al1.trim().to_string()))
        .set_address_zip_code(params.address_zip_code.map(|zip_code| zip_code.trim().to_string()))
        .set_address_city(params.address_city.map(|city| city.trim().to_string()))
        .set_address_country(is_address_provided.then(|| "FRANCE".to_string())),
    )
  }
}

impl ActiveModelEx {
  pub fn for_user(self, user_id: i32) -> Self {
    self.set_user_id(user_id)
  }

  pub async fn build(self, db: &impl TransactionTrait) -> Result<PractitionerCompany, MyErrors> {
    Ok(self.save(db).await?.try_into_model()?.into())
  }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CompanyParams {
  pub name: String,
  pub contact_name: String,
  #[validate(email(message = "invalid_email"))]
  pub contact_email: String,
  pub siret: Option<String>,
  pub address_line_1: Option<String>,
  pub address_zip_code: Option<String>,
  pub address_city: Option<String>,
}
