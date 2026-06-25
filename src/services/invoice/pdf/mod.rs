use oxidize_pdf::Page;

pub(in crate::services::invoice) mod company;
pub(in crate::services::invoice) mod patient;

/// Conversion constant: millimeters to points
/// PDF uses points (72 per inch), we use mm for convenience
pub(super) const MM_TO_POINTS: f64 = 2.834645669; // 72 / 25.4

pub(super) fn mm(value: f64) -> f64 {
  value * MM_TO_POINTS
}

pub(super) fn format_french_phone_number(phone_number: &str) -> String {
  let formatted = phone_number.replace("+33", "0");
  let digits: String = formatted.chars().filter(|c| c.is_ascii_digit()).collect();
  if digits.len() == 10 {
    format!(
      "{} {} {} {} {}",
      &digits[0..2],
      &digits[2..4],
      &digits[4..6],
      &digits[6..8],
      &digits[8..10]
    )
  } else {
    phone_number.to_string()
  }
}

pub(super) fn embed_signature_image(
  page: &mut Page,
  image_bytes: Vec<u8>,
  x_pt: f64,
  y_pt: f64,
) -> std::result::Result<(), String> {
  use image::codecs::jpeg::JpegEncoder;
  use image::ImageEncoder;
  use oxidize_pdf::graphics::Image;

  let img = image::load_from_memory(&image_bytes).map_err(|e| format!("Failed to decode image: {}", e))?;

  let rgb = img.to_rgb8();
  let (pixel_width, pixel_height) = rgb.dimensions();

  let max_width_pt = mm(60.0);
  let max_height_pt = mm(25.0);
  let aspect_ratio = pixel_width as f64 / pixel_height as f64;
  let (width_pt, height_pt) = if aspect_ratio > max_width_pt / max_height_pt {
    (max_width_pt, max_width_pt / aspect_ratio)
  } else {
    (max_height_pt * aspect_ratio, max_height_pt)
  };

  let mut jpg_data: Vec<u8> = Vec::new();
  JpegEncoder::new_with_quality(&mut jpg_data, 95)
    .write_image(rgb.as_raw(), pixel_width, pixel_height, image::ExtendedColorType::Rgb8)
    .map_err(|e| format!("Failed to encode as JPEG: {}", e))?;

  let image_obj = Image::from_jpeg_data(jpg_data).map_err(|e| format!("Failed to load signature image: {}", e))?;

  page.add_image("signature", image_obj);
  page
    .draw_image("signature", x_pt, y_pt - height_pt, width_pt, height_pt)
    .map_err(|e| format!("Failed to draw signature: {}", e))?;

  Ok(())
}
