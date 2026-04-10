#[derive(Clone, Debug)]
pub struct AppState {
  pub worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>,
}

impl AppState {
  pub fn new(worker_transmitter: tokio::sync::mpsc::Sender<WorkerJob>) -> Self {
    Self { worker_transmitter }
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
