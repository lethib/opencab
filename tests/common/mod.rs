use axum::Router;
use migration::{Migrator, MigratorTrait};
use opencab::{config::Config, middleware::context::AppState, router::create_router};
use sea_orm::{Database, DatabaseConnection, DatabaseTransaction, TransactionTrait};

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://loco:loco@localhost:5431/opencab_test";

pub struct TestApp {
  pub router: Router,
  pub db: DatabaseConnection,
  pub config: Config,
}

async fn connect_and_migrate() -> DatabaseConnection {
  unsafe {
    std::env::set_var("SSN_ENCRYPTION_KEY", "12345678901234567890123456789012");
    std::env::set_var("SSN_SALT_KEY", "bdd_test_salt_key_for_patients!!");
  }

  let db_url =
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
  let db = Database::connect(&db_url).await.unwrap();
  Migrator::up(&db, None).await.unwrap();
  db
}

pub async fn setup_tx() -> DatabaseTransaction {
  let db = connect_and_migrate().await;
  db.begin().await.unwrap()
}

pub async fn setup_http() -> TestApp {
  let db = connect_and_migrate().await;
  let config = Config::load("test").unwrap();
  let state = AppState {
    db: db.clone(),
    config: config.clone(),
  };
  let router = create_router(state);
  TestApp { router, db, config }
}
