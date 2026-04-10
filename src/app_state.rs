use crate::config::Config;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
  pub config: Arc<Config>,
  pub worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>,
}

impl AppState {
  pub fn new(config: Config, worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>) -> Self {
    Self {
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
