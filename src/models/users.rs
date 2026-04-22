use sea_orm::{
  prelude::*, ActiveValue, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
  TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
  auth::password,
  models::{
    _entities::{
      prelude::UserBusinessInformations, user_business_informations, user_practitioner_offices,
    },
    practitioner_offices, ModelError, ModelResult,
  },
  services,
};

pub use super::_entities::users::{self, ActiveModel, Model};

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginParams {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterParams {
  pub email: String,
  pub password: String,
  pub first_name: String,
  pub last_name: String,
  pub phone_number: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
  #[validate(email(message = "invalid email"))]
  pub email: String,
}

fn validate_model(model: &ActiveModel) -> Result<(), DbErr> {
  let validator = Validator {
    email: model.email.as_ref().to_owned(),
  };

  validator
    .validate()
    .map_err(|e| DbErr::Custom(format!("Validation error: {}", e)))
}

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::users::ActiveModel {
  async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    validate_model(&self)?;
    if insert {
      let mut this = self;
      this.pid = ActiveValue::Set(Uuid::new_v4());
      Ok(this)
    } else {
      Ok(self)
    }
  }
}

impl Model {
  pub fn full_name(&self) -> String {
    format!("{} {}", &self.first_name, &self.last_name)
  }

  pub async fn get_my_offices(
    &self,
    db: &DatabaseConnection,
  ) -> ModelResult<
    Vec<(
      practitioner_offices::Model,
      user_practitioner_offices::Model,
    )>,
  > {
    let offices = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::UserId.eq(self.id))
      .find_also_related(practitioner_offices::Entity)
      .all(db)
      .await?
      .into_iter()
      .filter_map(|(upo, office)| office.map(|o| (o, upo)))
      .collect();

    Ok(offices)
  }

  /// finds a user by the provided email
  ///
  /// # Errors
  ///
  /// When could not find user by the given token or DB query error
  pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Self> {
    let user = users::Entity::find()
      .filter(users::Column::Email.eq(email))
      .one(db)
      .await?;
    user.ok_or_else(|| ModelError::EntityNotFound)
  }

  /// finds a user by the provided pid
  ///
  /// # Errors
  ///
  /// When could not find user  or DB query error
  pub async fn find_by_pid(
    db: &DatabaseConnection,
    pid: &str,
  ) -> ModelResult<(Self, Option<user_business_informations::Model>)> {
    let parse_uuid = Uuid::parse_str(pid).map_err(|e| ModelError::Any(e.into()))?;
    let user = users::Entity::find()
      .filter(users::Column::Pid.eq(parse_uuid))
      .find_also_related(UserBusinessInformations)
      .one(db)
      .await?;
    user.ok_or_else(|| ModelError::EntityNotFound)
  }

  /// Verifies whether the provided plain password matches the hashed password
  ///
  /// # Errors
  ///
  /// when could not verify password
  #[must_use]
  pub fn verify_password(&self, password: &str) -> bool {
    password::verify_password(password, &self.password)
  }

  /// Asynchronously creates a user with a password and saves it to the
  /// database.
  ///
  /// # Errors
  ///
  /// When could not save the user into the DB
  pub async fn create_with_password(
    db: &DatabaseConnection,
    params: &RegisterParams,
  ) -> ModelResult<Self> {
    let txn = db.begin().await?;

    if users::Entity::find()
      .filter(users::Column::Email.eq(&params.email))
      .one(&txn)
      .await?
      .is_some()
    {
      return Err(ModelError::EntityAlreadyExists {});
    }

    let password_hash = password::hash_password(&params.password)
      .map_err(|e| ModelError::Any(format!("Password hash error: {}", e).into()))?;

    let access_key = services::user::generate_access_key();

    let user = users::ActiveModel {
      email: ActiveValue::set(params.email.to_string()),
      password: ActiveValue::set(password_hash),
      first_name: ActiveValue::set(params.first_name.clone()),
      last_name: ActiveValue::set(params.last_name.clone()),
      phone_number: ActiveValue::set(params.phone_number.clone()),
      access_key: ActiveValue::set(Some(access_key)),
      ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    Ok(user)
  }
}

impl ActiveModel {
  pub async fn enable_access(&mut self, db: &DatabaseConnection) -> ModelResult<()> {
    self.is_access_key_verified = ActiveValue::Set(true);
    self.clone().update(db).await?;

    Ok(())
  }

  pub async fn update_password(
    mut self,
    db: &DatabaseConnection,
    new_password: &str,
  ) -> ModelResult<Model> {
    let password_hash = password::hash_password(new_password)
      .map_err(|e| ModelError::Any(format!("Password hash error: {}", e).into()))?;

    self.password = ActiveValue::Set(password_hash);
    let updated_user = self.update(db).await?;

    Ok(updated_user)
  }
}
