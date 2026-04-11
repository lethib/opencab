use std::sync::OnceLock;

static LOCK: OnceLock<sea_orm::DatabaseConnection> = OnceLock::new();

pub struct DB {}
impl DB {
  pub fn init(db: sea_orm::DatabaseConnection) {
    LOCK.set(db).expect("Failed to initialize DB");
  }
  pub fn get() -> &'static sea_orm::DatabaseConnection {
    LOCK.get().expect("DB not initialized")
  }
}
