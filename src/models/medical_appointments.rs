use sea_orm::entity::prelude::*;

use crate::{
  auth::resource::Resource,
  models::{
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
    practitioner_offices,
  },
};

pub use super::_entities::medical_appointments::{ActiveModel, Entity, Model};

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

// implement your read-oriented logic here
impl Model {
  pub async fn practitioner_office<C: ConnectionTrait>(&self, db: &C) -> Result<practitioner_offices::Model, MyErrors> {
    self
      .find_related(practitioner_offices::Entity)
      .one(db)
      .await?
      .ok_or(UnexpectedError::should_not_happen().into())
  }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}

impl Resource for Model {
  async fn is_owned_by_user(&self, user_id: i32, _db: &DatabaseConnection) -> bool {
    self.user_id == user_id
  }

  fn resource_name(&self) -> String {
    "medical_appointments".to_string()
  }
}
