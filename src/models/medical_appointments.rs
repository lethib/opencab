use sea_orm::{entity::prelude::*, ActiveValue};

use crate::{
  auth::resource::Resource,
  db::DB,
  models::{
    _entities::sea_orm_active_enums::PaymentMethod,
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
    practitioner_offices,
  },
};

pub use super::_entities::medical_appointments::{ActiveModel, Entity, Model};

pub struct UpdateMedicalAppointmentParams {
  pub date: Date,
  pub price_in_cents: i32,
  pub practitioner_office_id: i32,
  pub payment_method: Option<PaymentMethod>,
}

pub struct CreateMedicalAppointmentParams {
  pub user_id: i32,
  pub patient_id: i32,
  pub practitioner_office_id: i32,
  pub date: Date,
  pub price_in_cents: i32,
  pub payment_method: Option<PaymentMethod>,
}

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
  pub async fn practitioner_office(&self) -> Result<practitioner_offices::Model, MyErrors> {
    self
      .find_related(practitioner_offices::Entity)
      .one(DB::get())
      .await?
      .ok_or(UnexpectedError::ShouldNotHappen.into())
  }
}

// implement your write-oriented logic here
impl ActiveModel {
  pub async fn update<T: ConnectionTrait>(
    mut self,
    db: &T,
    params: &UpdateMedicalAppointmentParams,
  ) -> Result<(), MyErrors> {
    self.date = ActiveValue::Set(params.date);
    self.practitioner_office_id = ActiveValue::Set(params.practitioner_office_id);
    self.price_in_cents = ActiveValue::Set(params.price_in_cents);
    self.payment_method = ActiveValue::Set(params.payment_method.clone());

    self.save(db).await?;

    Ok(())
  }

  pub async fn create<T: ConnectionTrait>(
    db: &T,
    params: &CreateMedicalAppointmentParams,
  ) -> Result<Model, MyErrors> {
    let created_medical_appointment = ActiveModel {
      user_id: ActiveValue::Set(params.user_id),
      patient_id: ActiveValue::Set(params.patient_id),
      practitioner_office_id: ActiveValue::Set(params.practitioner_office_id),
      date: ActiveValue::Set(params.date),
      price_in_cents: ActiveValue::Set(params.price_in_cents),
      payment_method: ActiveValue::Set(params.payment_method.clone()),
      ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(created_medical_appointment)
  }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}

impl Resource for Model {
  async fn is_owned_by_user(&self, user_id: i32) -> bool {
    self.user_id == user_id
  }

  fn resource_name(&self) -> String {
    "medical_appointments".to_string()
  }
}
