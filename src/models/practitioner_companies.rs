use sea_orm::{ActiveModelBehavior, ActiveValue, ConnectionTrait, DatabaseConnection, DbErr};

use crate::{
  auth::resource::Resource,
  models::_entities::practitioner_companies::{Model as PractitionerCompany, *},
};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    if !insert && self.updated_at.is_unchanged() {
      let mut this = self;
      this.updated_at = ActiveValue::Set(chrono::Utc::now().into());
      Ok(this)
    } else {
      Ok(self)
    }
  }
}

impl Resource for PractitionerCompany {
  async fn is_owned_by_user(&self, user_id: i32, _db: &DatabaseConnection) -> bool {
    self.user_id == user_id
  }

  fn resource_name(&self) -> String {
    "practitioner_companies".to_string()
  }
}
