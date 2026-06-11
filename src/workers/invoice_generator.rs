use axum::http::StatusCode;
use chrono::{Datelike, Duration};
use oxidize_pdf::graphics::Color;
use oxidize_pdf::text::Font;
use oxidize_pdf::{Document, Page};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;

use crate::db::DB;
use crate::models::{
  _entities::{
    company_interventions, patients, practitioner_companies, practitioner_offices,
    user_business_informations, users,
  },
  my_errors::{application_error::ApplicationError, MyErrors},
};
use crate::services::storage::StorageService;
use sea_orm::{prelude::Date, ColumnTrait, EntityTrait, QueryFilter};

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
    patient_ssn,
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

#[derive(Debug, Clone, Serialize)]
pub struct CompanyInvoiceArgs {
  pub intervention: company_interventions::Model,
  pub user: users::Model,
  pub business_info: user_business_informations::Model,
  pub company: practitioner_companies::Model,
  pub emission_date: Date,
  pub practitioner_office: practitioner_offices::Model,
  pub signature_data: Option<Vec<u8>>,
}

pub fn generate_company_invoice_pdf(args: &CompanyInvoiceArgs) -> Result<Vec<u8>, MyErrors> {
  create_company_invoice_pdf(args).map_err(|e| MyErrors {
    code: StatusCode::INTERNAL_SERVER_ERROR,
    msg: format!("Company invoice PDF creation failed: {}", e),
  })
}

fn format_euro(amount: f64) -> String {
  format!("€ {:.2}", amount).replace('.', ",")
}

fn format_siret(siret: &str) -> String {
  let digits: String = siret.chars().filter(|c| c.is_ascii_digit()).collect();
  if digits.len() == 14 {
    format!(
      "{} {} {} {}",
      &digits[0..3],
      &digits[3..6],
      &digits[6..9],
      &digits[9..14]
    )
  } else {
    siret.to_string()
  }
}

fn create_company_invoice_pdf(args: &CompanyInvoiceArgs) -> Result<Vec<u8>, String> {
  let mut doc = Document::new();
  doc.set_title("Facture");
  let mut page = Page::a4();

  // === CONSTANTS ===
  let page_height = mm(297.0);
  let margin_l = mm(20.0);
  let margin_r = mm(190.0);
  let col_right = mm(105.0); // FACTURÉ À column start

  let green = Color::hex("1B5E38");
  let light_gray_bg = Color::gray(0.95);
  let text_gray = Color::gray(0.45);
  let border_gray = Color::gray(0.80);

  // === CALCULATIONS ===
  let unit_price = args.intervention.unit_price_in_cents as f64 / 100.0;
  let quantity = args.intervention.quantity;
  let total_ht = unit_price * quantity as f64;
  let vat_rate = args
    .intervention
    .vat_rate_in_percent
    .to_f32()
    .unwrap_or(0.0);
  let vat_amount = total_ht * vat_rate as f64 / 100.0;
  let total_ttc = total_ht + vat_amount;
  let vat_display = if vat_rate.fract() == 0.0 {
    format!("{:.0} %", vat_rate)
  } else {
    format!("{} %", vat_rate)
  };

  let emission_date = args.emission_date;
  let due_date = emission_date + Duration::days(30);
  let invoice_number = format!("FAC-{}-{:03}", emission_date.year(), args.intervention.id);

  // ===========================
  // HEADER
  // ===========================
  // y_name = text baseline for the practitioner name (18pt)
  let y_name = page_height - mm(18.0);

  // Icon — favicon, vertically centred with 18pt text
  {
    use image::codecs::jpeg::JpegEncoder;
    use image::ImageEncoder;
    use oxidize_pdf::graphics::Image;
    const FAVICON_PNG: &[u8] = include_bytes!("../../frontend/public/favicon/apple-touch-icon.png");
    let img =
      ::image::load_from_memory(FAVICON_PNG).map_err(|e| format!("favicon decode: {}", e))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut jpg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpg, 95)
      .write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
      .map_err(|e| format!("favicon encode: {}", e))?;
    let image_obj = Image::from_jpeg_data(jpg).map_err(|e| format!("favicon image: {}", e))?;
    page.add_image("favicon", image_obj);
    page
      .draw_image("favicon", margin_l, y_name - mm(2.0), mm(7.0), mm(7.0))
      .map_err(|e| format!("favicon draw: {}", e))?;
  }

  // Practitioner name — starts 9.5 mm from left margin (2 mm gap after icon)
  page
    .text()
    .set_font(Font::HelveticaBold, 18.0)
    .set_fill_color(green)
    .at(margin_l + mm(9.5), y_name)
    .write(&args.user.full_name())
    .map_err(|e| format!("header name: {}", e))?;

  // "FACTURE" — right-aligned, top flush with icon top
  // Icon top = y_name - mm(2) + mm(7) = y_name + mm(5)
  // Helvetica Bold cap-height ≈ 718/1000 of font size
  let facture_cap_height = 30.0 * 0.718;
  let y_facture = y_name + mm(5.0) - facture_cap_height;
  let facture_w = oxidize_pdf::text::metrics::measure_text("FACTURE", Font::HelveticaBold, 30.0);
  page
    .text()
    .set_font(Font::HelveticaBold, 30.0)
    .set_fill_color(Color::black())
    .at(margin_r - facture_w, y_facture)
    .write("FACTURE")
    .map_err(|e| format!("FACTURE: {}", e))?;

  // Profession — 6 mm below name baseline
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_name - mm(6.0))
    .write(args.business_info.profession.to_french())
    .map_err(|e| format!("profession: {}", e))?;

  // Invoice number — right-aligned under FACTURE
  let invoice_label = format!("N°  {}", invoice_number);
  let invoice_label_w =
    oxidize_pdf::text::metrics::measure_text(&invoice_label, Font::Helvetica, 10.0);
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(text_gray)
    .at(margin_r - invoice_label_w, y_facture - mm(8.0))
    .write(&invoice_label)
    .map_err(|e| format!("invoice number: {}", e))?;

  let y_phone_number = y_name - mm(11.0);
  page
    .text()
    .set_font(Font::Helvetica, 9.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_phone_number)
    .write(&args.user.phone_number)
    .map_err(|e| format!("phone number: {}", e))?;

  let y_email = y_phone_number - mm(4.0);
  page
    .text()
    .set_font(Font::Helvetica, 9.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_email)
    .write(&args.user.email)
    .map_err(|e| format!("phone number: {}", e))?;

  // Header separator
  let y_sep1 = y_email - mm(6.0);
  page
    .graphics()
    .set_stroke_color(border_gray)
    .set_line_width(mm(0.3))
    .move_to(margin_l, y_sep1)
    .line_to(margin_r, y_sep1)
    .stroke();

  // ===========================
  // ÉMETTEUR / FACTURÉ À
  // ===========================
  let y_em = y_sep1 - mm(8.0);

  // Column labels
  page
    .text()
    .set_font(Font::Helvetica, 7.5)
    .set_fill_color(text_gray)
    .at(margin_l, y_em)
    .write("ÉMETTEUR")
    .map_err(|e| format!("emetteur label: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 7.5)
    .set_fill_color(text_gray)
    .at(col_right, y_em)
    .write("FACTURÉ À")
    .map_err(|e| format!("facture a label: {}", e))?;

  // Emetteur: name
  let y_name = y_em - mm(7.0);
  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_name)
    .write(&args.user.full_name())
    .map_err(|e| format!("emetteur name: {}", e))?;

  let mut y_address = y_name - mm(6.0);
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_address)
    .write(&args.practitioner_office.address_line_1)
    .map_err(|e| format!("company addr: {}", e))?;
  y_address -= mm(4.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_address)
    .write(&format!(
      "{} {}",
      args.practitioner_office.address_zip_code, args.practitioner_office.address_city
    ))
    .map_err(|e| format!("company city: {}", e))?;

  let y_siret = y_address - mm(6.0);
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_siret)
    .write(&format!(
      "SIRET {}",
      format_siret(&args.business_info.siret_number)
    ))
    .map_err(|e| format!("siret/adeli: {}", e))?;

  let y_rpps = y_siret - mm(6.0);
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(margin_l, y_rpps)
    .write(&format!("RPPS {}", &args.business_info.rpps_number))
    .map_err(|e| format!("siret/adeli: {}", e))?;

  // Company: name
  let y_company_name = y_em - mm(7.0);
  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(col_right, y_company_name)
    .write(&args.company.name)
    .map_err(|e| format!("company name: {}", e))?;

  // Company: address lines (each 4.0 mm apart)
  let mut company_y = y_company_name - mm(6.0);
  if let Some(ref addr) = args.company.address_line_1 {
    if !addr.is_empty() {
      page
        .text()
        .set_font(Font::Helvetica, 10.0)
        .set_fill_color(Color::black())
        .at(col_right, company_y)
        .write(addr)
        .map_err(|e| format!("company addr: {}", e))?;
      company_y -= mm(4.0);
    }
  }
  let city_line = match (&args.company.address_zip_code, &args.company.address_city) {
    (Some(zip), Some(city)) => format!("{} {}", zip, city),
    (None, Some(city)) => city.clone(),
    (Some(zip), None) => zip.clone(),
    _ => String::new(),
  };
  if !city_line.trim().is_empty() {
    page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_right, company_y)
      .write(&city_line)
      .map_err(|e| format!("company city: {}", e))?;
    company_y -= mm(6.0);
  }
  // Only show SIRET if non-empty
  if let Some(ref siret) = args.company.siret {
    if !siret.is_empty() {
      page
        .text()
        .set_font(Font::Helvetica, 10.0)
        .set_fill_color(Color::black())
        .at(col_right, company_y)
        .write(&format!("SIRET {}", format_siret(siret)))
        .map_err(|e| format!("company siret: {}", e))?;
    }
  }

  // ===========================
  // DATE BOX
  // ===========================
  let y_box_top = y_rpps - mm(9.0);
  let box_h = mm(20.0);

  // Rounded rectangle — cubic Bézier approximation of quarter-circles (k ≈ 0.5523)
  let bx = margin_l;
  let by = y_box_top - box_h;
  let bw = margin_r - margin_l;
  let bh = box_h;
  let br = mm(3.0);
  let bk = 0.5523_f64;
  page
    .graphics()
    .set_fill_color(light_gray_bg)
    .move_to(bx + br, by)
    .line_to(bx + bw - br, by)
    .curve_to(
      bx + bw - br + bk * br,
      by,
      bx + bw,
      by + br - bk * br,
      bx + bw,
      by + br,
    )
    .line_to(bx + bw, by + bh - br)
    .curve_to(
      bx + bw,
      by + bh - br + bk * br,
      bx + bw - br + bk * br,
      by + bh,
      bx + bw - br,
      by + bh,
    )
    .line_to(bx + br, by + bh)
    .curve_to(
      bx + br - bk * br,
      by + bh,
      bx,
      by + bh - br + bk * br,
      bx,
      by + bh - br,
    )
    .line_to(bx, by + br)
    .curve_to(bx, by + br - bk * br, bx + br - bk * br, by, bx + br, by)
    .close_path()
    .fill();

  let y_box_label = y_box_top - mm(4.5);
  let y_box_value = y_box_top - mm(13.0);

  // Date d'émission
  page
    .text()
    .set_font(Font::Helvetica, 7.5)
    .set_fill_color(text_gray)
    .at(margin_l + mm(5.0), y_box_label)
    .write("DATE D'ÉMISSION")
    .map_err(|e| format!("date label: {}", e))?;

  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(margin_l + mm(5.0), y_box_value)
    .write(&emission_date.format("%d/%m/%Y").to_string())
    .map_err(|e| format!("emission date: {}", e))?;

  // Échéance
  page
    .text()
    .set_font(Font::Helvetica, 7.5)
    .set_fill_color(text_gray)
    .at(mm(128.0), y_box_label)
    .write("ÉCHÉANCE")
    .map_err(|e| format!("echeance label: {}", e))?;

  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(mm(128.0), y_box_value)
    .write(&due_date.format("%d/%m/%Y").to_string())
    .map_err(|e| format!("due date: {}", e))?;

  // ===========================
  // TABLE
  // ===========================
  let y_table = y_box_top - box_h - mm(10.0);
  let col_desc = margin_l;
  let col_qty = mm(112.0);
  let col_pu = mm(128.0);
  let col_tva = mm(152.0);
  let col_total = mm(167.0);

  // Table headers
  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(text_gray)
    .at(col_desc, y_table)
    .write("DESCRIPTION")
    .map_err(|e| format!("desc header: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(text_gray)
    .at(col_qty, y_table)
    .write("QTÉ")
    .map_err(|e| format!("qty header: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(text_gray)
    .at(col_pu, y_table)
    .write("P.U. HT")
    .map_err(|e| format!("pu header: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(text_gray)
    .at(col_tva, y_table)
    .write("TVA")
    .map_err(|e| format!("tva header: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(text_gray)
    .at(col_total, y_table)
    .write("TOTAL HT")
    .map_err(|e| format!("total header: {}", e))?;

  // Separator under headers
  let y_line1 = y_table - mm(4.5);
  page
    .graphics()
    .set_stroke_color(border_gray)
    .set_line_width(mm(0.3))
    .move_to(margin_l, y_line1)
    .line_to(margin_r, y_line1)
    .stroke();

  // Item row
  let y_row = y_line1 - mm(9.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_desc, y_row)
    .write(&args.intervention.object)
    .map_err(|e| format!("object: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_qty, y_row)
    .write(&quantity.to_string())
    .map_err(|e| format!("qty value: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_pu, y_row)
    .write(&format_euro(unit_price))
    .map_err(|e| format!("pu value: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_tva, y_row)
    .write(&vat_display)
    .map_err(|e| format!("tva value: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_total, y_row)
    .write(&format_euro(total_ht))
    .map_err(|e| format!("total ht value: {}", e))?;

  // Separator after item
  let y_line2 = y_row - mm(11.0);
  page
    .graphics()
    .set_stroke_color(border_gray)
    .set_line_width(mm(0.3))
    .move_to(margin_l, y_line2)
    .line_to(margin_r, y_line2)
    .stroke();

  // ===========================
  // TOTALS
  // ===========================
  let y_total_ht = y_line2 - mm(9.0);

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(mm(118.0), y_total_ht)
    .write("Total HT")
    .map_err(|e| format!("total ht label: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_total, y_total_ht)
    .write(&format_euro(total_ht))
    .map_err(|e| format!("total ht amount: {}", e))?;

  let y_vat = y_total_ht - mm(6.0);
  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(mm(118.0), y_vat)
    .write("TVA")
    .map_err(|e| format!("vat label: {}", e))?;

  page
    .text()
    .set_font(Font::Helvetica, 10.0)
    .set_fill_color(Color::black())
    .at(col_total, y_vat)
    .write(&format_euro(vat_amount))
    .map_err(|e| format!("vat amount: {}", e))?;

  // Divider above TTC
  let y_line3 = y_total_ht - mm(7.0);
  page
    .graphics()
    .set_stroke_color(Color::black())
    .set_line_width(mm(0.4))
    .move_to(mm(108.0), y_line3)
    .line_to(margin_r, y_line3)
    .stroke();

  let y_ttc = y_line3 - mm(8.0);

  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(mm(118.0), y_ttc)
    .write("Total TTC")
    .map_err(|e| format!("total ttc label: {}", e))?;

  page
    .text()
    .set_font(Font::HelveticaBold, 11.0)
    .set_fill_color(Color::black())
    .at(col_total, y_ttc)
    .write(&format_euro(total_ttc))
    .map_err(|e| format!("total ttc amount: {}", e))?;

  // ===========================
  // SIGNATURE (bottom-left)
  // ===========================
  if let Some(ref sig_bytes) = args.signature_data {
    let sig_x_mm = (margin_l + mm(5.0)) / MM_TO_POINTS;
    let sig_y_mm = (y_total_ht - mm(15.0)) / MM_TO_POINTS;
    match embed_signature_image(&mut page, sig_bytes, sig_x_mm, sig_y_mm) {
      Ok(_) => tracing::info!("Embedded signature in company invoice"),
      Err(e) => tracing::warn!("Failed to embed signature in company invoice: {}", e),
    }
  }

  // ===========================
  // FOOTER
  // ===========================
  let y_footer = mm(25.0);
  page
    .graphics()
    .set_stroke_color(border_gray)
    .set_line_width(mm(0.3))
    .move_to(margin_l, y_footer)
    .line_to(margin_r, y_footer)
    .stroke();

  if vat_rate != 0.0 {
    page
      .text()
      .set_font(Font::Helvetica, 8.0)
      .set_fill_color(Color::gray(0.45))
      .at(margin_l, y_footer - mm(5.0))
      .write("TVA non applicable — art. 261 4° 1° du CGI")
      .map_err(|e| format!("footer legal: {}", e))?;
  }

  page
    .text()
    .set_font(Font::Helvetica, 8.0)
    .set_fill_color(Color::gray(0.45))
    .at(mm(118.0), y_footer - mm(5.0))
    .write("Paiement par virement — 30 jours")
    .map_err(|e| format!("footer payment: {}", e))?;

  doc.add_page(page);
  doc.to_bytes().map_err(|e| format!("PDF bytes: {}", e))
}

/// Create a simple invoice PDF matching the provided template
#[allow(clippy::too_many_arguments)]
fn create_modern_invoice_pdf(
  user: &users::Model,
  business_info: &user_business_informations::Model,
  patient: &patients::Model,
  patient_ssn: Option<String>,
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

  if let Some(patient_ssn) = patient_ssn {
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
  }

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
