use super::_entities::user_business_informations::{ActiveModel, Entity, Model};
use crate::models::_entities::sea_orm_active_enums::Profession;
use crate::models::user_business_informations;
use crate::validators::business_information::{validate_rpps_number, validate_siret_number};
use sea_orm::{entity::prelude::*, ActiveEnum, ActiveValue};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateBusinessInformation {
  pub rpps_number: String,
  pub adeli_number: Option<String>,
  pub siret_number: String,
  pub profession: String,
}

#[derive(Deserialize)]
pub struct BankingInformationParams {
  pub beneficiary_name: String,
  pub iban: String,
  pub bic: String,
}

impl CreateBusinessInformation {
  pub fn profession_enum(&self) -> Result<Profession, DbErr> {
    let value = sea_orm::sea_query::Enum {
      type_name: Profession::name().inner().into(),
      value: self.profession.clone().into(),
    };
    Profession::try_from_value(&value).map_err(|_| DbErr::Custom(format!("Invalid profession value: {}", self.profession)))
  }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    let mut this = self;

    // if this.rpps_number matches an ActiveValue::Set pattern (i.e when the values changes),
    // it extracts the inner value and bind it to a variable we call rpps
    if let ActiveValue::Set(ref rpps) = this.rpps_number {
      if !validate_rpps_number(rpps) {
        return Err(DbErr::Custom("RPPS_number_not_valid".to_string()));
      }
    }

    if let ActiveValue::Set(ref siret) = this.siret_number {
      if !validate_siret_number(siret) {
        return Err(DbErr::Custom("SIRET_number_not_valid".to_string()));
      }
    }

    if !insert && this.updated_at.is_unchanged() {
      this.updated_at = ActiveValue::Set(chrono::Utc::now().into());
    }

    Ok(this)
  }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
  pub async fn create<T: ConnectionTrait>(
    db: &T,
    params: &CreateBusinessInformation,
    concerned_user_id: &i32,
  ) -> Result<Model, DbErr> {
    user_business_informations::ActiveModel {
      user_id: ActiveValue::Set(*concerned_user_id),
      rpps_number: ActiveValue::Set(params.rpps_number.clone()),
      siret_number: ActiveValue::Set(params.siret_number.clone()),
      adeli_number: ActiveValue::Set(params.adeli_number.clone()),
      profession: ActiveValue::Set(params.profession_enum()?),
      ..Default::default()
    }
    .insert(db)
    .await
  }

  pub async fn save_banking_information<T: ConnectionTrait>(
    mut self,
    db: &T,
    params: BankingInformationParams,
  ) -> Result<(), DbErr> {
    self.beneficiary_name = ActiveValue::Set(Some(params.beneficiary_name));
    self.iban = ActiveValue::Set(Some(params.iban.trim().to_string()));
    self.bic = ActiveValue::Set(Some(params.bic.trim().to_string()));

    self.save(db).await?;

    Ok(())
  }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
