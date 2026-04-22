use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://loco:loco@localhost:5431/opencab_test";

// Callers must use #[serial] to prevent races on the env vars set below.
pub async fn setup_db() -> DatabaseConnection {
  unsafe {
    std::env::set_var("SSN_ENCRYPTION_KEY", "12345678901234567890123456789012");
    std::env::set_var("SSN_SALT_KEY", "bdd_test_salt_key_for_patients!!");
  }

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
