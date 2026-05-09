use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};
use serde::Deserialize;
use validator::Validate;

use crate::{
  auth::resource::Resource,
  models::{
    _entities::practitioner_companies,
    my_errors::{application_error::ApplicationError, MyErrors},
  },
  validators::address::is_address_valid,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CompanyParams {
  pub name: String,
  #[validate(email(message = "invalid_email"))]
  pub contact_email: String,
  pub address_line_1: Option<String>,
  pub address_zip_code: Option<String>,
}

impl practitioner_companies::ActiveModel {
  pub async fn create<T: ConnectionTrait>(
    db: &T,
    owner_id: i32,
    params: &CompanyParams,
  ) -> Result<practitioner_companies::Model, MyErrors> {
    params.validate()?;
    validate_address_params(params)?;

    let is_address_provided = params.address_line_1.is_some();

    Ok(
      Self {
        name: ActiveValue::Set(params.name.trim().to_string()),
        user_id: ActiveValue::Set(owner_id),
        contact_email: ActiveValue::Set(params.contact_email.trim().to_string()),
        address_line_1: ActiveValue::Set(
          params
            .address_line_1
            .as_ref()
            .map(|al1| al1.trim().to_string()),
        ),
        address_zip_code: ActiveValue::Set(
          params
            .address_zip_code
            .as_ref()
            .map(|zip_code| zip_code.trim().to_string()),
        ),
        address_country: ActiveValue::Set(is_address_provided.then_some("FRANCE".to_string())),
        ..Default::default()
      }
      .insert(db)
      .await?,
    )
  }

  pub async fn update<T: ConnectionTrait>(
    mut self,
    db: &T,
    params: &CompanyParams,
  ) -> Result<(), MyErrors> {
    params.validate()?;
    validate_address_params(params)?;
    let is_address_provided = params.address_line_1.is_some();

    self.name = ActiveValue::Set(params.name.trim().to_string());
    self.contact_email = ActiveValue::Set(params.contact_email.trim().to_string());
    self.address_line_1 = ActiveValue::Set(
      params
        .address_line_1
        .as_ref()
        .map(|al1| al1.trim().to_string()),
    );
    self.address_zip_code = ActiveValue::Set(
      params
        .address_zip_code
        .as_ref()
        .map(|zip_code| zip_code.trim().to_string()),
    );
    self.address_country = ActiveValue::Set(is_address_provided.then_some("FRANCE".to_string()));

    self.save(db).await?;

    Ok(())
  }
}

fn validate_address_params(params: &CompanyParams) -> Result<(), MyErrors> {
  if let (Some(address_line_1), Some(zip_code)) = (
    params.address_line_1.as_ref(),
    params.address_zip_code.as_ref(),
  ) {
    if !is_address_valid(address_line_1, zip_code) {
      return Err(ApplicationError::UnprocessableEntity.into());
    }
  }

  Ok(())
}

impl Resource for practitioner_companies::Model {
  async fn is_owned_by_user(&self, user_id: i32) -> bool {
    self.user_id == user_id
  }

  fn resource_name(&self) -> String {
    "practitioner_companies".to_string()
  }
}
