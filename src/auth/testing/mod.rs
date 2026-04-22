pub mod user_factory;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://loco:loco@localhost:5431/opencab_test";

pub async fn setup_db() -> DatabaseConnection {
  let db_url =
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
  let db = Database::connect(&db_url).await.unwrap();
  Migrator::up(&db, None).await.unwrap();
  db.execute_unprepared(
    "TRUNCATE TABLE medical_appointments, user_practitioner_offices,
     user_business_informations, patients, practitioner_offices, users
     RESTART IDENTITY CASCADE",
  )
  .await
  .unwrap();
  db
}
