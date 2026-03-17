use crate::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
  pub db: DatabaseConnection,
  pub config: Arc<Config>,
  pub worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>,
}

impl AppState {
  pub fn new(
    db: DatabaseConnection,
    config: Config,
    worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>,
  ) -> Self {
    Self {
      db,
      config: Arc::new(config),
      worker_transmitter,
    }
  }
}

// Worker job enum for all background tasks
#[derive(Debug, Clone)]
pub enum WorkerJob {
  Email(crate::workers::mailer::args::EmailArgs),
  AppointmentExport(
    crate::workers::appointments_export::AppointmentExtractorArgs,
    AppState,
  ),
  AccountabilityGeneration(
    crate::workers::appointments_export::AccountabilityGenerationArgs,
    AppState,
  ),
}
