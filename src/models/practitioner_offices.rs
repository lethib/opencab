pub use super::_entities::practitioner_offices::{ActiveModel, Entity, Model};
use crate::{
  auth::resource::Resource,
  db::DB,
  models::{
    _entities::{practitioner_offices, user_practitioner_offices},
    my_errors::{application_error::ApplicationError, MyErrors},
  },
  validators::address::is_address_valid,
};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    if !insert && self.updated_at.is_unchanged() {
      let mut this = self;
      this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
      Ok(this)
    } else {
      Ok(self)
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PractitionerOfficeParams {
  pub name: String,
  pub address_line_1: String,
  pub address_zip_code: String,
  pub address_city: String,
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
  pub async fn create<T: ConnectionTrait>(
    db: &T,
    params: &PractitionerOfficeParams,
  ) -> Result<Model, MyErrors> {
    if !is_address_valid(&params.address_line_1, &params.address_zip_code) {
      return Err(ApplicationError::UnprocessableEntity.into());
    }

    return Ok(
      practitioner_offices::ActiveModel {
        name: ActiveValue::Set(params.name.trim().to_string()),
        address_line_1: ActiveValue::Set(params.address_line_1.trim().to_string()),
        address_zip_code: ActiveValue::Set(params.address_zip_code.trim().to_string()),
        address_city: ActiveValue::Set(params.address_city.trim().to_string()),
        address_country: ActiveValue::Set("FRANCE".to_string()),
        ..Default::default()
      }
      .insert(db)
      .await?,
    );
  }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}

impl Resource for Model {
  async fn is_owned_by_user(&self, user_id: i32) -> bool {
    let result = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::PractitionerOfficeId.eq(self.id))
      .filter(user_practitioner_offices::Column::UserId.eq(user_id))
      .one(DB::get())
      .await;

    match result {
      Ok(association) => association.is_some(),
      Err(_) => false,
    }
  }

  fn resource_name(&self) -> String {
    "practitioner_office".to_string()
  }
}
