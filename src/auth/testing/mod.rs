pub mod user_factory;

use migration::{Migrator, MigratorTrait};
use sea_orm::{
  ConnectionTrait, Database, DatabaseConnection, DatabaseTransaction, TransactionTrait,
};
use tokio::sync::OnceCell;

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://loco:loco@localhost:5431/opencab_test";

// Migrations must run exactly once per test process. Running `Migrator::up`
// concurrently from multiple tests races on creating the `seaql_migrations`
// table and panics with a `pg_type` unique-constraint violation.
static MIGRATED: OnceCell<()> = OnceCell::const_new();

async fn connect() -> DatabaseConnection {
  let db_url =
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
  Database::connect(&db_url).await.unwrap()
}

async fn ensure_migrated() {
  MIGRATED
    .get_or_init(|| async {
      let db = connect().await;
      Migrator::up(&db, None).await.unwrap();
    })
    .await;
}

pub async fn setup_tx() -> DatabaseTransaction {
  ensure_migrated().await;
  let db = connect().await;
  db.begin().await.unwrap()
}

pub async fn setup_db() -> DatabaseConnection {
  ensure_migrated().await;
  let db = connect().await;
  db.execute_unprepared(
    "TRUNCATE TABLE medical_appointments, user_practitioner_offices,
     user_business_informations, patients, practitioner_offices, users
     RESTART IDENTITY CASCADE",
  )
  .await
  .unwrap();
  db
}
