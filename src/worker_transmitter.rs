use std::sync::OnceLock;
use tokio::sync::mpsc;

pub static LOCK: OnceLock<mpsc::Sender<WorkerJob>> = OnceLock::new();

pub struct WorkerTransmitter {}
impl WorkerTransmitter {
  pub fn get() -> &'static mpsc::Sender<WorkerJob> {
    LOCK.get().expect("WorkerTransmitter not initialized")
  }
}

// Worker job enum for all background tasks
#[derive(Debug, Clone)]
pub enum WorkerJob {
  Email(crate::workers::mailer::args::EmailArgs),
  AppointmentExport(crate::workers::appointments_export::AppointmentExtractorArgs),
  AccountabilityGeneration(crate::workers::appointments_export::AccountabilityGenerationArgs),
}
