use crate::models::{_entities::user_practitioner_offices, my_errors::MyErrors};

pub use super::_entities::user_practitioner_offices::{ActiveModel, Entity, Model};
use sea_orm::{entity::prelude::*, ActiveValue};

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

pub struct CreateLinkParams {
  pub user_id: i32,
  pub revenue_share_percentage: Decimal,
  pub practitioner_office_id: i32,
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
  pub async fn create<T: ConnectionTrait>(db: &T, params: &CreateLinkParams) -> Result<Model, MyErrors> {
    return Ok(
      user_practitioner_offices::ActiveModel {
        user_id: ActiveValue::Set(params.user_id),
        practitioner_office_id: ActiveValue::Set(params.practitioner_office_id),
        revenue_share_percentage: ActiveValue::Set(params.revenue_share_percentage),
        ..Default::default()
      }
      .insert(db)
      .await?,
    );
  }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
