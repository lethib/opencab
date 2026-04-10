use opencab::db;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app_state;
mod auth;
mod config;
mod controllers;
mod middleware;
mod models;
mod router;
mod services;
mod validators;
mod views;
mod workers;

use app_state::AppState;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::from_filename(".env.local").ok();

  let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
  let config = Config::load(&environment).expect("Failed to load configuration");

  setup_logging(&config.logger.level, &config.logger.format);

  tracing::info!(
    "Starting opencab application (environment: {})",
    environment
  );

  let mut db_options = sea_orm::ConnectOptions::new(&config.database.url);
  db_options.sqlx_logging(config.database.enable_logging);

  let db = sea_orm::Database::connect(db_options)
    .await
    .expect("Failed to connect to database");
  tracing::info!("Connected to database");

  db::LOCK.set(db).expect("Failed to load DB");

  let (worker_transmitter, worker_receiver) = workers::create_worker_channel();
  let state = AppState::new(config.clone(), worker_transmitter);

  let worker_config = state.config.clone();
  tokio::spawn(async move {
    workers::start_worker_pool(worker_receiver, worker_config).await;
  });

  tracing::info!("Worker pool started");

  let app = router::create_router(state.clone());

  let addr = format!("{}:{}", config.server.binding, config.server.port);
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .unwrap_or_else(|_| panic!("Failed to bind to address: {}", addr));

  tracing::info!(
    "Server listening on {}:{}",
    config.server.host,
    config.server.port
  );

  // Run server with graceful shutdown
  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");

  Ok(())
}

fn setup_logging(level: &str, format: &str) {
  let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| format!("opencab={},tower_http={},sqlx=info", level, level).into());

  let registry = tracing_subscriber::registry().with(env_filter);

  if format == "json" {
    registry
      .with(tracing_subscriber::fmt::layer().json())
      .init();
  } else {
    registry.with(tracing_subscriber::fmt::layer()).init();
  }
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("Failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("Failed to install signal handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }

  tracing::info!("Shutdown signal received, starting graceful shutdown");
}
