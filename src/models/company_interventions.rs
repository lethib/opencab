use sea_orm::{ActiveModelBehavior, ActiveValue, ConnectionTrait, DatabaseConnection, DbErr, ModelTrait};

use crate::{
  auth::resource::Resource,
  models::{
    _entities::{company_interventions, practitioner_companies, prelude},
    my_errors::{application_error::ApplicationError, MyErrors},
  },
};

pub const ALLOWED_VAT_VALUES: [f32; 4] = [0.0, 5.5, 10.0, 20.0];

impl company_interventions::Model {
  pub async fn company(&self, db: &DatabaseConnection) -> Result<practitioner_companies::Model, MyErrors> {
    self
      .find_related(prelude::PractitionerCompanies)
      .one(db)
      .await?
      .ok_or(ApplicationError::not_found().into())
  }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for company_interventions::ActiveModel {
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

impl Resource for company_interventions::Model {
  async fn is_owned_by_user(&self, user_id: i32, _db: &DatabaseConnection) -> bool {
    self.practitioner_id == user_id
  }

  fn resource_name(&self) -> String {
    "company_intervention".to_string()
  }
}
