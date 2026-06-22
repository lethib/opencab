use std::future::Future;

use sea_orm::DatabaseConnection;

pub trait Resource {
  fn is_owned_by_user(&self, user_id: i32, db: &DatabaseConnection) -> impl Future<Output = bool>;

  fn resource_name(&self) -> String;
}
