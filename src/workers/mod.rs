use crate::worker_transmitter::WorkerJob;
use tokio::sync::mpsc;

pub mod appointments_export;
pub mod downloader;
pub mod invoice_generator;
pub mod mailer;

const WORKER_CHANNEL_SIZE: usize = 100;

/// Create a new worker channel for background job processing
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
          WorkerJob::AppointmentExport(args) => {
            appointments_export::process_appointment_extraction(args).await
          }
          WorkerJob::AccountabilityGeneration(args) => {
            appointments_export::process_accountability_generation(args).await
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
