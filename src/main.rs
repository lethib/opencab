use opencab::{
  config::Config,
  db::DB,
  router,
  workers::{self, WorkerTransmitter},
};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::from_filename(".env.local").ok();

  let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
  let config = Config::load(&environment).expect("Failed to load configuration");
  Config::init(config);

  setup_logging(&Config::get().logger.level, &Config::get().logger.format);

  tracing::info!(
    "Starting opencab application (environment: {})",
    environment
  );

  let mut db_options = sea_orm::ConnectOptions::new(&Config::get().database.url);
  db_options.sqlx_logging(Config::get().database.enable_logging);

  let db = sea_orm::Database::connect(db_options)
    .await
    .expect("Failed to connect to database");
  tracing::info!("Connected to database");

  DB::init(db);

  let (worker_transmitter, worker_receiver) = workers::create_worker_channel();
  WorkerTransmitter::init(worker_transmitter);

  tokio::spawn(async move {
    workers::start_worker_pool(worker_receiver).await;
  });

  tracing::info!("Worker pool started");

  let app = router::create_router();

  let addr = format!(
    "{}:{}",
    Config::get().server.binding,
    Config::get().server.port
  );
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .unwrap_or_else(|_| panic!("Failed to bind to address: {}", addr));

  tracing::info!(
    "Server listening on {}:{}",
    Config::get().server.host,
    Config::get().server.port
  );

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
