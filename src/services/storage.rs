use crate::models::my_errors::{unexpected_error::UnexpectedError, MyErrors};
use axum::http::StatusCode;
use reqwest::Client;
use std::env;
use tracing::{error, info};

/// Service for handling Supabase storage operations
pub struct StorageService {
  client: Client,
  supabase_url: String,
  supabase_key: String,
  bucket_name: String,
}

impl StorageService {
  /// Create a new StorageService instance
  pub fn new() -> Result<Self, MyErrors> {
    let supabase_url = env::var("SUPABASE_URL").map_err(|_| MyErrors {
      code: StatusCode::INTERNAL_SERVER_ERROR,
      msg: "SUPABASE_URL environment variable not set".to_string(),
    })?;

    let supabase_key = env::var("SUPABASE_SERVICE_ROLE_KEY").map_err(|_| MyErrors {
      code: StatusCode::INTERNAL_SERVER_ERROR,
      msg: "SUPABASE_SERVICE_ROLE_KEY environment variable not set".to_string(),
    })?;

    let bucket_name = env::var("SUPABASE_SIGNATURE_BUCKET").unwrap_or_else(|_| "signatures".to_string());

    let client = Client::new();

    Ok(Self {
      client,
      supabase_url,
      supabase_key,
      bucket_name,
    })
  }

  pub fn signature_url(&self, signature_filename: &str) -> String {
    format!(
      "{}/storage/v1/object/public/{}/{}",
      self.supabase_url, self.bucket_name, signature_filename
    )
  }

  /// Fetch a signature image from Supabase storage
  ///
  /// # Arguments
  /// * `user_id` - The user ID to fetch the signature for
  ///
  /// # Returns
  /// * `Result<Vec<u8>, MyErrors>` - The image bytes or an error
  pub async fn fetch_signature(&self, signature_file_name: &str) -> Result<Vec<u8>, MyErrors> {
    let url = self.signature_url(signature_file_name);

    info!("Fetching signature from: {}", url);

    let response = self
      .client
      .get(&url)
      .header("Authorization", format!("Bearer {}", self.supabase_key))
      .send()
      .await
      .map_err(|e| {
        error!("Failed to fetch signature: {}", e);
        MyErrors {
          code: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!("Failed to fetch signature from storage: {}", e),
        }
      })?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_default();

      return match status {
        reqwest::StatusCode::NOT_FOUND => Err(MyErrors {
          code: StatusCode::NOT_FOUND,
          msg: format!("Signature not found for user {}", signature_file_name),
        }),
        _ => {
          error!("Supabase storage error {}: {}", status, error_text);
          Err(MyErrors {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Storage service error: {} - {}", status, error_text),
          })
        }
      };
    }

    let image_bytes = response.bytes().await.map_err(|e| {
      error!("Failed to read signature bytes: {}", e);
      MyErrors {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Failed to read signature data: {}", e),
      }
    })?;

    info!("Successfully fetched signature: {} bytes", image_bytes.len());
    Ok(image_bytes.to_vec())
  }

  /// Upload a signature image to Supabase storage
  ///
  /// # Arguments
  /// * `signature_data` - The image bytes to upload
  /// * `filename` - The filename to use for the signature
  /// * `content_type` - The MIME type of the file (e.g., "image/png", "image/jpeg")
  ///
  /// # Returns
  /// * `Result<(), MyErrors>` - Success or an error
  pub async fn upload_signature(&self, signature_data: &[u8], filename: &str, content_type: &str) -> Result<(), MyErrors> {
    let url = format!("{}/storage/v1/object/{}/{}", self.supabase_url, self.bucket_name, filename);

    info!(
      "Uploading signature to: /storage/v1/object/{}/{}, size: {} bytes, content_type: {}",
      self.bucket_name,
      filename,
      signature_data.len(),
      content_type
    );

    let response = self
      .client
      .post(&url)
      .header("Authorization", format!("Bearer {}", self.supabase_key))
      .header("Content-Type", content_type)
      .body(signature_data.to_vec())
      .send()
      .await
      .map_err(|e| {
        error!("Failed to send upload request: {}", e);
        UnexpectedError::new("failed_to_upload_signature".to_string())
      })?;

    if response.status().is_success() {
      info!("Successfully uploaded signature: {}", filename);
      Ok(())
    } else {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_default();
      error!("Failed to upload to Supabase storage ({}): {}", status, error_text);
      Err(UnexpectedError::new(format!("Storage upload failed: {} - {}", status, error_text)).into())
    }
  }
}
