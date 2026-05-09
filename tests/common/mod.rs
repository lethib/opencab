use axum::Router;
use migration::{Migrator, MigratorTrait};
use opencab::{config::Config, db::DB, router::create_router};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use std::sync::LazyLock;

// Shared runtime for all HTTP integration tests. sqlx returns connections to
// the pool via tokio::spawn; if each test used its own runtime (as #[tokio::test]
// does), those spawned release tasks would be cancelled on runtime shutdown and
// the pool's semaphore would never be incremented back, exhausting the pool.
pub static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
});

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
     user_business_informations, patients, practitioner_offices,
     practitioner_companies, users
     RESTART IDENTITY CASCADE",
  )
  .await
  .unwrap();

  db
}

// Tracks whether the DB singleton has been initialized for HTTP tests.
static HTTP_DB_SETUP: std::sync::OnceLock<()> = std::sync::OnceLock::new();

// Initializes global singletons on the first call (idempotent), then truncates
// all tables. Callers must use #[serial] to prevent races on env vars and
// the shared DB singleton. After calling this, use DB::get() in tests.
pub async fn setup_http() -> Router {
  unsafe {
    std::env::set_var("SSN_ENCRYPTION_KEY", "12345678901234567890123456789012");
    std::env::set_var("SSN_SALT_KEY", "bdd_test_salt_key_for_patients!!");
  }

  if HTTP_DB_SETUP.get().is_none() {
    let db_url = std::env::var("TEST_DATABASE_URL")
      .unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
    let db = Database::connect(&db_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    DB::init(db);
    Config::init(Config::load("test").unwrap());
    let _ = HTTP_DB_SETUP.set(());
  }

  DB::get()
    .execute_unprepared(
      "TRUNCATE TABLE medical_appointments, user_practitioner_offices,
       user_business_informations, patients, practitioner_offices,
       practitioner_companies, users
       RESTART IDENTITY CASCADE",
    )
    .await
    .unwrap();

  create_router()
}
