use crate::{
  auth::resource::Resource,
  models::{
    _entities::patients,
    my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
  },
  services::crypto::Crypto,
  validators::address::is_address_valid,
};

pub use super::_entities::patients::{ActiveModel, Entity, Model};
use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePatientParams {
  pub first_name: String,
  pub last_name: String,
  pub ssn: String,
  pub address_line_1: String,
  pub address_zip_code: String,
  pub address_city: String,
  pub email: Option<String>,
}

// Encryption utilities for SSN
// implement your read-oriented logic here
impl Model {
  fn encrypt_ssn(ssn: &str) -> Result<String, MyErrors> {
    Crypto::encrypt(ssn)
  }

  fn hash_ssn(ssn: &str) -> Result<String, MyErrors> {
    let salt =
      std::env::var("SSN_SALT_KEY").map_err(|err| UnexpectedError::new(err.to_string()))?;
    Crypto::hash(ssn, &salt)
  }

  pub fn decrypt_ssn(&self) -> Result<String, MyErrors> {
    Crypto::decrypt(&self.ssn)
  }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    let mut this = self;
    if insert {
      this.pid = ActiveValue::Set(Uuid::new_v4());
    } else if this.updated_at.is_unchanged() {
      this.updated_at = ActiveValue::Set(chrono::Utc::now().into())
    }
    Ok(this)
  }
}

// implement your write-oriented logic here
impl ActiveModel {
  pub async fn create<T: ConnectionTrait>(
    db: &T,
    params: &CreatePatientParams,
    linked_to_user_id: i32,
  ) -> Result<Model, MyErrors> {
    if !is_address_valid(&params.address_line_1, &params.address_zip_code) {
      return Err(ApplicationError::UnprocessableEntity.into());
    }

    let ssn_encrypted = Model::encrypt_ssn(&params.ssn)?;
    let ssn_hashed = Model::hash_ssn(&params.ssn)?;

    return Ok(
      patients::ActiveModel {
        first_name: ActiveValue::Set(params.first_name.clone()),
        last_name: ActiveValue::Set(params.last_name.clone()),
        email: ActiveValue::Set(params.email.clone()),
        ssn: ActiveValue::Set(ssn_encrypted),
        hashed_ssn: ActiveValue::Set(ssn_hashed),
        address_line_1: ActiveValue::Set(params.address_line_1.clone()),
        address_zip_code: ActiveValue::Set(params.address_zip_code.clone()),
        address_city: ActiveValue::Set(params.address_city.clone()),
        address_country: ActiveValue::Set("FRANCE".to_string()),
        user_id: ActiveValue::Set(linked_to_user_id),
        ..Default::default()
      }
      .insert(db)
      .await?,
    );
  }

  pub async fn update<T: ConnectionTrait>(
    db: &T,
    patient_id: i32,
    params: &CreatePatientParams,
  ) -> Result<(), MyErrors> {
    let mut patient = Entity::find_by_id(patient_id)
      .one(db)
      .await?
      .expect("Patient not found")
      .into_active_model();

    if !is_address_valid(&params.address_line_1, &params.address_zip_code) {
      return Err(ApplicationError::UnprocessableEntity.into());
    }

    patient.first_name = ActiveValue::Set(params.first_name.trim().to_string());
    patient.last_name = ActiveValue::Set(params.last_name.trim().to_string());
    patient.email = ActiveValue::Set(params.email.as_ref().map(|e| e.trim().to_string()));
    patient.address_line_1 = ActiveValue::Set(params.address_line_1.trim().to_string());
    patient.address_zip_code = ActiveValue::Set(params.address_zip_code.trim().to_string());
    patient.address_city = ActiveValue::Set(params.address_city.trim().to_string());

    patient.update(db).await?;

    Ok(())
  }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}

impl Resource for Model {
  async fn is_owned_by_user(&self, user_id: i32) -> bool {
    self.user_id == user_id
  }

  fn resource_name(&self) -> String {
    "patient".to_string()
  }
}
