use crate::{db::DB, models::_entities::user_business_informations::ActiveModel};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DbErr, IntoActiveModel, ModelTrait};

use crate::models::{
  _entities::user_business_informations, user_business_informations::CreateBusinessInformation,
  users,
};

pub async fn save_business_information(
  params: &CreateBusinessInformation,
  concerned_user: &users::Model,
) -> Result<(), DbErr> {
  let business_info = concerned_user
    .find_related(user_business_informations::Entity)
    .one(DB::get())
    .await?;

  match business_info {
    Some(business_information) => {
      let mut business_information = business_information.into_active_model();

      business_information.rpps_number = Set(params.rpps_number.clone());
      business_information.siret_number = Set(params.siret_number.clone());
      business_information.adeli_number = Set(params.adeli_number.clone());
      business_information.profession = Set(params.profession_enum()?);

      business_information.update(DB::get()).await?;
      Ok(())
    }
    None => {
      ActiveModel::create(DB::get(), params, &concerned_user.id).await?;
      Ok(())
    }
  }
}

pub fn check_access_key(concerned_user: &users::Model, access_key: String) -> bool {
  match &concerned_user.access_key {
    None => false,
    Some(user_access_key) => *user_access_key == access_key,
  }
}

/// Generate a random access key in the format XXX-XXX-XXX-XXX
pub fn generate_access_key() -> String {
  use rand::Rng;
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  const GROUP_LENGTH: usize = 3;
  const NUM_GROUPS: usize = 4;

  let mut rng = rand::thread_rng();

  let mut key = String::with_capacity(GROUP_LENGTH * NUM_GROUPS + NUM_GROUPS - 1);

  for i in 0..NUM_GROUPS {
    for _ in 0..GROUP_LENGTH {
      let idx = rng.gen_range(0..CHARSET.len());
      key.push(CHARSET[idx] as char);
    }

    if i < NUM_GROUPS - 1 {
      key.push('-');
    }
  }

  key
}
