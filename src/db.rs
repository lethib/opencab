use std::sync::OnceLock;

pub static LOCK: OnceLock<sea_orm::DatabaseConnection> = OnceLock::new();

pub struct DB {}
impl DB {
  pub fn get() -> &'static sea_orm::DatabaseConnection {
    LOCK.get().expect("DB not initialized")
  }
}
