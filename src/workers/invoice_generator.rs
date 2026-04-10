use axum::http::StatusCode;
use oxidize_pdf::graphics::Color;
use oxidize_pdf::text::Font;
use oxidize_pdf::{Document, Page};
use serde::Serialize;

use crate::models::{
  _entities::{patients, practitioner_offices, user_business_informations, users},
  my_errors::{application_error::ApplicationError, MyErrors},
};
use crate::services::storage::StorageService;
use sea_orm::{prelude::Date, ColumnTrait, EntityTrait, QueryFilter};
use crate::db::DB;

/// Conversion constant: millimeters to points
/// PDF uses points (72 per inch), we use mm for convenience
const MM_TO_POINTS: f64 = 2.834645669; // 72 / 25.4

/// Helper function to convert millimeters to points
fn mm(value: f64) -> f64 {
  value * MM_TO_POINTS
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceGeneratorArgs {
  pub patient: patients::Model,
  pub user: users::Model,
  pub amount: f32,
  pub invoice_date: Date,
  pub practitioner_office: practitioner_offices::Model,
  pub is_duplicate: bool,
}

/// Generate an invoice PDF based on the French invoice template
pub async fn generate_invoice_pdf(
  args: &InvoiceGeneratorArgs,
) -> std::result::Result<Vec<u8>, MyErrors> {
  // Initialize storage service for signature fetching
  let storage_service = match StorageService::new() {
    Ok(service) => Some(service),
    Err(e) => {
      tracing::warn!(
        "Storage service unavailable: {}. Continuing without signature.",
        e
      );
      None
    }
  };

  // Fetch business information separately
  let business_info = user_business_informations::Entity::find()
    .filter(user_business_informations::Column::UserId.eq(args.user.id))
    .one(DB::get())
    .await?
    .ok_or_else(|| MyErrors {
      code: StatusCode::BAD_REQUEST,
      msg: "User business information not found".to_string(),
    })?;

  // Try to fetch signature if storage service is available
  let signature_data = match &storage_service {
    Some(service) => match service
      .fetch_signature(
        business_info
          .signature_file_name
          .as_ref()
          .ok_or(ApplicationError::UnprocessableEntity)?,
      )
      .await
    {
      Ok(data) => {
        tracing::info!("Successfully fetched signature for user {}", args.user.id);
        Some(data)
      }
      Err(e) => {
        tracing::warn!(
          "Failed to fetch signature for user {}: {}. Continuing without signature.",
          args.user.id,
          e
        );
        None
      }
    },
    None => None,
  };

  // Decrypt patient SSN
  let patient_ssn = args.patient.decrypt_ssn()?;

  // Generate PDF
  let pdf_data = create_modern_invoice_pdf(
    &args.user,
    &business_info,
    &args.patient,
    &patient_ssn,
    &args.amount,
    &args.invoice_date,
    &args.practitioner_office,
    signature_data.as_deref(),
    args.is_duplicate,
  )
  .map_err(|e| MyErrors {
    code: StatusCode::INTERNAL_SERVER_ERROR,
    msg: format!("PDF creation failed: {}", e),
  })?;

  Ok(pdf_data)
}

/// Embed a signature image into the PDF page
///
/// # Arguments
/// * `page` - Mutable reference to the PDF page
/// * `image_bytes` - The image bytes (JPG/PNG)
/// * `x_mm` - X position in millimeters
/// * `y_mm` - Y position in millimeters (from bottom)
///
/// # Returns
/// * `Result<(), String>` - Success or error message
fn embed_signature_image(
  page: &mut Page,
  image_bytes: &[u8],
  x_mm: f64,
  y_mm: f64,
) -> std::result::Result<(), String> {
  use oxidize_pdf::graphics::Image;

  // Load and decode the image (auto-detect format)
  let img =
    ::image::load_from_memory(image_bytes).map_err(|e| format!("Failed to decode image: {}", e))?;

  // Convert to RGB if needed
  let rgb_img = img.to_rgb8();
  let (width, height) = rgb_img.dimensions();

  tracing::info!("Loaded signature image: {}x{} pixels", width, height);

  // Calculate aspect ratio and target dimensions
  let max_width_mm = 60.0; // Maximum width for signature
  let max_height_mm = 30.0; // Maximum height for signature

  let aspect_ratio = width as f64 / height as f64;
  let (target_width_mm, target_height_mm) = if aspect_ratio > max_width_mm / max_height_mm {
    // Width-constrained
    (max_width_mm, max_width_mm / aspect_ratio)
  } else {
    // Height-constrained
    (max_height_mm * aspect_ratio, max_height_mm)
  };

  tracing::info!(
    "Target signature dimensions: {:.2}x{:.2} mm",
    target_width_mm,
    target_height_mm
  );

  // Convert image to JPEG format for oxidizePdf
  let mut jpg_data = Vec::new();
  {
    use image::codecs::jpeg::JpegEncoder;
    use image::ImageEncoder;
    let encoder = JpegEncoder::new_with_quality(&mut jpg_data, 95);
    encoder
      .write_image(
        rgb_img.as_raw(),
        width,
        height,
        image::ExtendedColorType::Rgb8,
      )
      .map_err(|e| format!("Failed to encode as JPEG: {}", e))?;
  }

  // Create image from JPEG data
  let image_obj =
    Image::from_jpeg_data(jpg_data).map_err(|e| format!("Failed to create JPEG image: {}", e))?;

  // Add image to page resources
  let image_name = "signature";
  page.add_image(image_name, image_obj);

  // Calculate final position (adjust Y for image height)
  let final_y_mm = y_mm - target_height_mm;

  // Draw the image on the page
  page
    .draw_image(
      image_name,
      mm(x_mm),
      mm(final_y_mm),
      mm(target_width_mm),
      mm(target_height_mm),
    )
    .map_err(|e| format!("Failed to draw image: {}", e))?;

  tracing::info!(
    "Successfully embedded signature image at ({:.2}, {:.2}) mm with size {:.2}x{:.2} mm",
    x_mm,
    y_mm,
    target_width_mm,
    target_height_mm
  );

  Ok(())
}

/// Create a simple invoice PDF matching the provided template
#[allow(clippy::too_many_arguments)]
fn create_modern_invoice_pdf(
  user: &users::Model,
  business_info: &user_business_informations::Model,
  patient: &patients::Model,
  patient_ssn: &str,
  amount: &f32,
  invoice_date: &Date,
  practitioner_office: &practitioner_offices::Model,
  signature_data: Option<&[u8]>,
  is_duplicate: bool,
) -> std::result::Result<Vec<u8>, String> {
  // Create PDF document
  let mut doc = Document::new();
  doc.set_title("Note d'honoraires acquittée");

  // Create A4 page (210mm x 297mm = 595 x 842 points)
  let mut page = Page::a4();

  // Page setup constants
  let page_height = mm(297.0);
  let margin = mm(25.0);
  let mut y_position = page_height - margin - mm(10.0);

  // === DUPLICATA WATERMARK ===
  if is_duplicate {
    page
      .text()
      .set_font(Font::HelveticaBold, 70.0)
      .set_fill_color(Color::gray(0.82))
      .set_character_spacing(mm(2.5))
      .at(mm(22.0), mm(148.5))
      .write("DUPLICATA")
      .map_err(|e| format!("Failed to write watermark: {}", e))?;
    page
      .text()
      .set_fill_color(Color::black())
      .set_character_spacing(0.0);
  }

  // === HEADER SECTION ===
  // Practitioner name with title on same line, separated by dash
  let full_name = format!(
    "{} – {}",
    &user.full_name(),
    business_info.profession.to_french()
  );
  page
    .text()
    .set_font(Font::HelveticaBold, 14.0)
    .at(margin, y_position)
    .write(&full_name)
    .map_err(|e| format!("Failed to write practitioner name: {}", e))?;
  y_position -= mm(12.0);

  // Professional numbers with consistent formatting
  if let Some(ref adeli) = business_info.adeli_number {
    page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(margin, y_position)
      .write(&format!("N° Adeli : {}", adeli))
      .map_err(|e| format!("Failed to write Adeli number: {}", e))?;
    y_position -= mm(5.0);
  }

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&format!("N°RPPS : {}", business_info.rpps_number))
    .map_err(|e| format!("Failed to write RPPS number: {}", e))?;
  y_position -= mm(5.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&format!("N°SIRET : {}", business_info.siret_number))
    .map_err(|e| format!("Failed to write SIRET number: {}", e))?;
  y_position -= mm(12.0);

  // Address
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&practitioner_office.address_line_1)
    .map_err(|e| format!("Failed to write address line 1: {}", e))?;
  y_position -= mm(5.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&format!(
      "{} {}",
      practitioner_office.address_zip_code, practitioner_office.address_city,
    ))
    .map_err(|e| format!("Failed to write address city: {}", e))?;
  y_position -= mm(8.0);

  // Contact info
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&format!("Tel : {}", &user.phone_number))
    .map_err(|e| format!("Failed to write phone number: {}", e))?;
  y_position -= mm(8.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .at(margin, y_position)
    .write(&user.email)
    .map_err(|e| format!("Failed to write email: {}", e))?;
  y_position -= mm(30.0);

  // === INVOICE TITLE - CENTERED ===
  let title = "Note d'honoraires acquittée";
  page
    .text()
    .set_font(Font::HelveticaBold, 20.0)
    .at(mm(60.0), y_position)
    .write(title)
    .map_err(|e| format!("Failed to write title: {}", e))?;
  y_position -= mm(30.0);

  // === PATIENT INFORMATION ===
  // Patient name
  let patient_full_name = format!("{} {}", patient.last_name, patient.first_name);
  let full_text = format!("Reçu de : {}", patient_full_name);

  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(margin, y_position)
    .write(&full_text)
    .map_err(|e| format!("Failed to write patient name: {}", e))?;

  // Draw underline only for "Reçu de :"
  let underline_y = y_position - mm(1.0);
  page
    .graphics()
    .set_stroke_color(Color::black())
    .set_line_width(mm(0.3))
    .move_to(margin, underline_y)
    .line_to(margin + mm(17.0), underline_y)
    .stroke();

  y_position -= mm(12.0);

  // Social security number with box
  let ssn_y = y_position;
  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(margin, y_position)
    .write(&format!("Numéro de sécurité sociale : {}", patient_ssn))
    .map_err(|e| format!("Failed to write SSN: {}", e))?;

  // Draw box around SSN field
  let box_x = margin - mm(2.0);
  let box_y = ssn_y - mm(3.0);
  let box_width = mm(185.0) - box_x;
  let box_height = mm(8.0);

  page
    .graphics()
    .set_stroke_color(Color::black())
    .set_line_width(mm(0.5))
    .rect(box_x, box_y, box_width, box_height)
    .stroke();

  y_position -= mm(18.0);

  // Address with box
  let addr_y = y_position;
  let address_text = format!(
    "Adresse : {} – {} {}",
    patient.address_line_1, patient.address_zip_code, patient.address_city
  );
  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(margin, y_position)
    .write(&address_text)
    .map_err(|e| format!("Failed to write patient address: {}", e))?;

  // Draw box around address field
  let addr_box_x = margin - mm(2.0);
  let addr_box_y = addr_y - mm(3.0);
  let addr_box_width = mm(185.0) - addr_box_x;
  let addr_box_height = mm(8.0);

  page
    .graphics()
    .set_stroke_color(Color::black())
    .set_line_width(mm(0.5))
    .rect(addr_box_x, addr_box_y, addr_box_width, addr_box_height)
    .stroke();

  y_position -= mm(18.0);

  let full_text = format!("Honoraire : {:.2}€", amount);

  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(margin, y_position)
    .write(&full_text)
    .map_err(|e| format!("Failed to write amount: {}", e))?;

  // Draw underline only for "Honoraire :"
  let underline_text = "Honoraire :";
  let text_width = underline_text.len() as f64 * 2.5; // Rough estimate: 2.5mm per character at 11pt
  let underline_y = y_position - mm(1.0);

  page
    .graphics()
    .set_stroke_color(Color::black())
    .set_line_width(mm(0.3))
    .move_to(margin, underline_y)
    .line_to(margin + mm(text_width), underline_y)
    .stroke();
  y_position -= mm(35.0);

  // === DATE AND SIGNATURE ===
  let invoice_date_str = invoice_date.format("%d/%m/%Y").to_string();
  let date_location = format!(
    "Fait à {}, le {}",
    practitioner_office.address_city, invoice_date_str
  );

  // Right align date
  let date_x = mm(230.0) - margin - mm(85.0);
  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(date_x, y_position)
    .write(&date_location)
    .map_err(|e| format!("Failed to write date: {}", e))?;

  // Signature section - left aligned
  let sig_x_mm = 30.0;

  // Try to embed signature image if available
  if let Some(sig_bytes) = signature_data {
    match embed_signature_image(&mut page, sig_bytes, sig_x_mm, y_position / MM_TO_POINTS) {
      Ok(_) => {
        tracing::info!("Successfully embedded signature image");
      }
      Err(e) => {
        tracing::warn!(
          "Failed to embed signature image: {}. Using text fallback.",
          e
        );
      }
    }
  }

  y_position -= mm(30.0);

  // Practitioner name below signature
  page
    .text()
    .set_font(Font::Helvetica, 11.0)
    .at(date_x + mm(20.0), y_position)
    .write(&user.full_name())
    .map_err(|e| format!("Failed to write practitioner name at bottom: {}", e))?;

  // Add the page to the document
  doc.add_page(page);

  // Generate PDF bytes
  doc
    .to_bytes()
    .map_err(|e| format!("Failed to generate PDF: {}", e))
}
