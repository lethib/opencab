use sea_orm::DatabaseConnection;
use std::sync::OnceLock;
use tokio::sync::mpsc;

pub mod appointments_export;
pub mod downloader;
pub mod mailer;

const WORKER_CHANNEL_SIZE: usize = 100;

static LOCK: OnceLock<mpsc::Sender<WorkerJob>> = OnceLock::new();

pub struct WorkerTransmitter {}
impl WorkerTransmitter {
  pub fn init(wt: mpsc::Sender<WorkerJob>) {
    LOCK.set(wt).expect("Failed to initiallize WorkerTransmitter");
  }
  pub fn get() -> &'static mpsc::Sender<WorkerJob> {
    LOCK.get().expect("WorkerTransmitter not initialized")
  }
}

#[derive(Debug, Clone)]
pub enum WorkerJob {
  Email(mailer::args::EmailArgs),
  AppointmentExport(appointments_export::AppointmentExtractorArgs, DatabaseConnection),
  AccountabilityGeneration(appointments_export::AccountabilityGenerationArgs, DatabaseConnection),
}

pub fn create_worker_channel() -> (mpsc::Sender<WorkerJob>, mpsc::Receiver<WorkerJob>) {
  mpsc::channel(WORKER_CHANNEL_SIZE)
}

/// Start the worker pool with the specified number of workers
/// Each worker will continuously process jobs from the channel
pub async fn start_worker_pool(mut rx: mpsc::Receiver<WorkerJob>) {
  tokio::spawn(async move {
    while let Some(job) = rx.recv().await {
      // Spawn a task for each job
      tokio::spawn(async move {
        tracing::debug!("Processing job");

        let result = match job {
          WorkerJob::Email(args) => mailer::worker::process_email(args).await,
          WorkerJob::AppointmentExport(args, db) => appointments_export::process_appointment_extraction(args, &db).await,
          WorkerJob::AccountabilityGeneration(args, db) => {
            appointments_export::process_accountability_generation(args, &db).await
          }
        };

        if let Err(e) = result {
          tracing::error!("Worker job failed: {:?}", e);
        } else {
          tracing::debug!("Job completed successfully");
        }
      });
    }

    tracing::info!("Worker pool stopped");
  });
}
