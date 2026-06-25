use chrono::{Datelike, Duration, NaiveDate};
use oxidize_pdf::{Color, Document, Font, Image, Page};
use rust_decimal::prelude::ToPrimitive;

use crate::{
  models::{
    _entities::{company_interventions, practitioner_companies, practitioner_offices, user_business_informations},
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
    users::users,
  },
  services::invoice::pdf::{embed_signature_image, format_french_phone_number, mm},
};

pub(in crate::services::invoice) struct CompanyPdfArgs {
  pub intervention: company_interventions::Model,
  pub user: users::Model,
  pub business_info: user_business_informations::Model,
  pub company: practitioner_companies::Model,
  pub emission_date: NaiveDate,
  pub practitioner_office: practitioner_offices::Model,
  pub signature_data: Option<Vec<u8>>,
}

pub(in crate::services::invoice) struct CompanyInvoiceGenerator {
  pub args: CompanyPdfArgs,
  doc: Document,
  page: Page,
  y_position: f64,
  margin_l: f64,
  margin_r: f64,
}

fn format_euro(amount: f64) -> String {
  format!("€ {:.2}", amount).replace('.', ",")
}

fn format_siret(siret: &str) -> String {
  let digits: String = siret.chars().filter(|c| c.is_ascii_digit()).collect();
  if digits.len() == 14 {
    format!("{} {} {} {}", &digits[0..3], &digits[3..6], &digits[6..9], &digits[9..14])
  } else {
    siret.to_string()
  }
}

impl CompanyInvoiceGenerator {
  pub(in crate::services::invoice) fn new(args: CompanyPdfArgs) -> Self {
    let (doc, page, margin_l, margin_r, y_position) = Self::setup_document();
    Self {
      args,
      doc,
      page,
      y_position,
      margin_l,
      margin_r,
    }
  }

  pub(in crate::services::invoice) fn build(mut self) -> Result<Self, MyErrors> {
    self.build_header()?;
    self.y_position -= mm(8.0);

    self.build_parties()?;
    self.y_position -= mm(9.0);

    self.build_date_box()?;
    self.y_position -= mm(10.0);

    self.build_table()?;
    self.y_position -= mm(9.0);

    self.build_totals()?;

    self.build_footer()?;
    Ok(self)
  }

  #[allow(clippy::wrong_self_convention)]
  pub(in crate::services::invoice) fn to_bytes(mut self) -> Result<Vec<u8>, MyErrors> {
    self.doc.add_page(self.page);
    self.doc.to_bytes().map_err(|e| UnexpectedError::new(e).into())
  }

  fn build_header(&mut self) -> Result<(), MyErrors> {
    let invoice_number = format!("FAC-{}-{:03}", self.args.emission_date.year(), self.args.intervention.id);

    // Favicon, name, "FACTURE", profession, and invoice number are all anchored
    // to the name baseline (self.y_position); the cursor only advances afterwards.
    let favicon = Self::load_favicon()?;
    self.page.add_image("favicon", favicon);
    self
      .page
      .draw_image("favicon", self.margin_l, self.y_position - mm(2.0), mm(7.0), mm(7.0))
      .map_err(UnexpectedError::new)?;

    let green = Color::hex("1B5E38");
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 18.0)
      .set_fill_color(green)
      .at(self.margin_l + mm(9.5), self.y_position)
      .write(&self.args.user.full_name())
      .map_err(UnexpectedError::new)?;

    // "FACTURE" — right-aligned, top flush with icon top
    // Icon top = y_position - mm(2) + mm(7) = y_position + mm(5)
    // Helvetica Bold cap-height ≈ 718/1000 of font size
    let facture_cap_height = 30.0 * 0.718;
    let y_facture = self.y_position + mm(5.0) - facture_cap_height;
    let facture_w = oxidize_pdf::text::metrics::measure_text("FACTURE", Font::HelveticaBold, 30.0);
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 30.0)
      .set_fill_color(Color::black())
      .at(self.margin_r - facture_w, y_facture)
      .write("FACTURE")
      .map_err(UnexpectedError::new)?;

    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position - mm(6.0))
      .write(self.args.business_info.profession.to_french())
      .map_err(UnexpectedError::new)?;

    let text_gray = Color::gray(0.45);
    let invoice_label = format!("N°  {}", invoice_number);
    let invoice_label_w = oxidize_pdf::text::metrics::measure_text(&invoice_label, Font::Helvetica, 10.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(text_gray)
      .at(self.margin_r - invoice_label_w, y_facture - mm(8.0))
      .write(&invoice_label)
      .map_err(UnexpectedError::new)?;

    self.y_position -= mm(11.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 9.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&format_french_phone_number(&self.args.user.phone_number))
      .map_err(UnexpectedError::new)?;

    self.y_position -= mm(4.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 9.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&self.args.user.email)
      .map_err(UnexpectedError::new)?;

    self.y_position -= mm(6.0);
    self
      .page
      .graphics()
      .set_stroke_color(Color::gray(0.80))
      .set_line_width(mm(0.3))
      .move_to(self.margin_l, self.y_position)
      .line_to(self.margin_r, self.y_position)
      .stroke();

    Ok(())
  }

  fn build_parties(&mut self) -> Result<(), MyErrors> {
    let col_right = mm(105.0);
    let text_gray = Color::gray(0.45);

    // Column labels
    self
      .page
      .text()
      .set_font(Font::Helvetica, 7.5)
      .set_fill_color(text_gray)
      .at(self.margin_l, self.y_position)
      .write("ÉMETTEUR")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 7.5)
      .set_fill_color(text_gray)
      .at(col_right, self.y_position)
      .write("FACTURÉ À")
      .map_err(UnexpectedError::new)?;

    // Names — both columns share the same row
    self.y_position -= mm(7.0);
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&self.args.user.full_name())
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(col_right, self.y_position)
      .write(&self.args.company.name)
      .map_err(UnexpectedError::new)?;

    // Address line 1 — right column is optional and tracked separately
    self.y_position -= mm(6.0);
    let mut company_y = self.y_position;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&self.args.practitioner_office.address_line_1)
      .map_err(UnexpectedError::new)?;
    if let Some(ref addr) = self.args.company.address_line_1 {
      if !addr.is_empty() {
        self
          .page
          .text()
          .set_font(Font::Helvetica, 10.0)
          .set_fill_color(Color::black())
          .at(col_right, company_y)
          .write(addr)
          .map_err(UnexpectedError::new)?;
        company_y -= mm(4.0);
      }
    }

    // City
    self.y_position -= mm(4.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&format!(
        "{} {}",
        self.args.practitioner_office.address_zip_code, self.args.practitioner_office.address_city
      ))
      .map_err(UnexpectedError::new)?;
    let city_line = match (&self.args.company.address_zip_code, &self.args.company.address_city) {
      (Some(zip), Some(city)) => format!("{} {}", zip, city),
      (None, Some(city)) => city.clone(),
      (Some(zip), None) => zip.clone(),
      _ => String::new(),
    };
    if !city_line.trim().is_empty() {
      self
        .page
        .text()
        .set_font(Font::Helvetica, 10.0)
        .set_fill_color(Color::black())
        .at(col_right, company_y)
        .write(&city_line)
        .map_err(UnexpectedError::new)?;
      company_y -= mm(6.0);
    }

    // SIRET (left) / optional SIRET (right)
    self.y_position -= mm(6.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&format!("SIRET : {}", format_siret(&self.args.business_info.siret_number)))
      .map_err(UnexpectedError::new)?;
    if let Some(ref siret) = self.args.company.siret {
      if !siret.is_empty() {
        self
          .page
          .text()
          .set_font(Font::Helvetica, 10.0)
          .set_fill_color(Color::black())
          .at(col_right, company_y)
          .write(&format!("SIRET : {}", format_siret(siret)))
          .map_err(UnexpectedError::new)?;
      }
    }

    // RPPS (left only)
    self.y_position -= mm(6.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(self.margin_l, self.y_position)
      .write(&format!("RPPS : {}", &self.args.business_info.rpps_number))
      .map_err(UnexpectedError::new)?;

    if let Some(adeli_number) = &self.args.business_info.adeli_number {
      self.y_position -= mm(6.0);
      self
        .page
        .text()
        .set_font(Font::Helvetica, 10.0)
        .set_fill_color(Color::black())
        .at(self.margin_l, self.y_position)
        .write(&format!("ADELI : {}", adeli_number))
        .map_err(UnexpectedError::new)?;
    }

    Ok(())
  }

  fn build_date_box(&mut self) -> Result<(), MyErrors> {
    let due_date = self.args.emission_date + Duration::days(30);
    let box_h = mm(20.0);
    self.y_position -= mm(5.0);

    // Rounded rectangle — cubic Bézier approximation of quarter-circles (k ≈ 0.5523)
    let bx = self.margin_l;
    let by = self.y_position - box_h;
    let bw = self.margin_r - self.margin_l;
    let bh = box_h;
    let br = mm(3.0);
    let bk = 0.5523_f64;
    self
      .page
      .graphics()
      .set_fill_color(Color::gray(0.95))
      .move_to(bx + br, by)
      .line_to(bx + bw - br, by)
      .curve_to(bx + bw - br + bk * br, by, bx + bw, by + br - bk * br, bx + bw, by + br)
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
      .curve_to(bx + br - bk * br, by + bh, bx, by + bh - br + bk * br, bx, by + bh - br)
      .line_to(bx, by + br)
      .curve_to(bx, by + br - bk * br, bx + br - bk * br, by, bx + br, by)
      .close_path()
      .fill();

    // Labels and values are at fixed offsets from the box top (self.y_position)
    self
      .page
      .text()
      .set_font(Font::Helvetica, 7.5)
      .set_fill_color(Color::gray(0.45))
      .at(self.margin_l + mm(5.0), self.y_position - mm(4.5))
      .write("DATE D'ÉMISSION")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(self.margin_l + mm(5.0), self.y_position - mm(13.0))
      .write(&self.args.emission_date.format("%d/%m/%Y").to_string())
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 7.5)
      .set_fill_color(Color::gray(0.45))
      .at(mm(128.0), self.y_position - mm(4.5))
      .write("ÉCHÉANCE")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(mm(128.0), self.y_position - mm(13.0))
      .write(&due_date.format("%d/%m/%Y").to_string())
      .map_err(UnexpectedError::new)?;

    self.y_position -= box_h;
    Ok(())
  }

  fn build_table(&mut self) -> Result<(), MyErrors> {
    let (unit_price, quantity, total_ht, vat_rate) = self.compute_amounts();
    let vat_display = if vat_rate.fract() == 0.0 {
      format!("{:.0} %", vat_rate)
    } else {
      format!("{} %", vat_rate)
    };

    let col_desc = self.margin_l;
    let col_qty = mm(112.0);
    let col_pu = mm(128.0);
    let col_tva = mm(152.0);
    let col_total = mm(167.0);

    self.y_position -= mm(5.0);

    // Headers
    for (col, label) in [
      (col_desc, "DESCRIPTION"),
      (col_qty, "QTÉ"),
      (col_pu, "P.U. HT"),
      (col_tva, "TVA"),
      (col_total, "TOTAL HT"),
    ] {
      self
        .page
        .text()
        .set_font(Font::Helvetica, 8.0)
        .set_fill_color(Color::gray(0.45))
        .at(col, self.y_position)
        .write(label)
        .map_err(UnexpectedError::new)?;
    }

    // Separator under headers
    self.y_position -= mm(4.5);
    self
      .page
      .graphics()
      .set_stroke_color(Color::gray(0.80))
      .set_line_width(mm(0.3))
      .move_to(self.margin_l, self.y_position)
      .line_to(self.margin_r, self.y_position)
      .stroke();

    // Item row
    self.y_position -= mm(9.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_desc, self.y_position)
      .write(&self.args.intervention.object)
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_qty, self.y_position)
      .write(&quantity.to_string())
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_pu, self.y_position)
      .write(&format_euro(unit_price))
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_tva, self.y_position)
      .write(&vat_display)
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_total, self.y_position)
      .write(&format_euro(total_ht))
      .map_err(UnexpectedError::new)?;

    // Separator after item
    self.y_position -= mm(11.0);
    self
      .page
      .graphics()
      .set_stroke_color(Color::gray(0.80))
      .set_line_width(mm(0.3))
      .move_to(self.margin_l, self.y_position)
      .line_to(self.margin_r, self.y_position)
      .stroke();

    Ok(())
  }

  fn build_totals(&mut self) -> Result<(), MyErrors> {
    let (_, _, total_ht, vat_rate) = self.compute_amounts();
    let vat_amount = total_ht * vat_rate as f64 / 100.0;
    let total_ttc = total_ht + vat_amount;

    // Signature is anchored to the Total HT baseline; save before cursor moves.
    let y_sig = self.y_position - mm(25.0);

    let col_total = mm(167.0);

    // Total HT
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(mm(118.0), self.y_position)
      .write("Total HT")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_total, self.y_position)
      .write(&format_euro(total_ht))
      .map_err(UnexpectedError::new)?;

    // TVA
    self.y_position -= mm(6.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(mm(118.0), self.y_position)
      .write("TVA")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .set_fill_color(Color::black())
      .at(col_total, self.y_position)
      .write(&format_euro(vat_amount))
      .map_err(UnexpectedError::new)?;

    // Divider above TTC
    self.y_position -= mm(7.0);
    self
      .page
      .graphics()
      .set_stroke_color(Color::black())
      .set_line_width(mm(0.4))
      .move_to(mm(108.0), self.y_position)
      .line_to(self.margin_r, self.y_position)
      .stroke();

    // Total TTC
    self.y_position -= mm(8.0);
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(mm(118.0), self.y_position)
      .write("Total TTC")
      .map_err(UnexpectedError::new)?;
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 11.0)
      .set_fill_color(Color::black())
      .at(col_total, self.y_position)
      .write(&format_euro(total_ttc))
      .map_err(UnexpectedError::new)?;

    if let Some(sig_bytes) = self.args.signature_data.take() {
      match embed_signature_image(&mut self.page, sig_bytes, self.margin_l + mm(15.0), y_sig) {
        Ok(_) => tracing::info!("Embedded signature in company invoice"),
        Err(e) => tracing::warn!("Failed to embed signature in company invoice: {}", e),
      }
    }

    Ok(())
  }

  fn build_footer(&mut self) -> Result<(), MyErrors> {
    let border_gray = Color::gray(0.80);

    let y_footer = mm(25.0);
    self
      .page
      .graphics()
      .set_stroke_color(border_gray)
      .set_line_width(mm(0.3))
      .move_to(self.margin_l, y_footer)
      .line_to(self.margin_r, y_footer)
      .stroke();

    self
      .page
      .text()
      .set_font(Font::Helvetica, 8.0)
      .set_fill_color(Color::gray(0.45))
      .at(self.margin_l, y_footer - mm(5.0))
      .write("TVA non applicable — art. 261 4° 1° du CGI")
      .map_err(UnexpectedError::new)?;

    self
      .page
      .text()
      .set_font(Font::Helvetica, 8.0)
      .set_fill_color(Color::gray(0.45))
      .at(mm(118.0), y_footer - mm(5.0))
      .write("Paiement par virement — 30 jours")
      .map_err(UnexpectedError::new)?;

    Ok(())
  }

  fn compute_amounts(&self) -> (f64, i32, f64, f32) {
    let unit_price = self.args.intervention.unit_price_in_cents as f64 / 100.0;
    let quantity = self.args.intervention.quantity;
    let total_ht = unit_price * quantity as f64;
    let vat_rate = self.args.intervention.vat_rate_in_percent.to_f32().unwrap_or(0.0);

    (unit_price, quantity, total_ht, vat_rate)
  }

  fn load_favicon() -> Result<Image, MyErrors> {
    use image::codecs::jpeg::JpegEncoder;
    use image::ImageEncoder;
    use oxidize_pdf::graphics::Image;

    const FAVICON_PNG: &[u8] = include_bytes!("../../../../frontend/public/favicon/apple-touch-icon.png");
    let img = ::image::load_from_memory(FAVICON_PNG).map_err(UnexpectedError::new)?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut jpg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpg, 95)
      .write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
      .map_err(UnexpectedError::new)?;

    Image::from_jpeg_data(jpg).map_err(|e| UnexpectedError::new(e).into())
  }

  fn setup_document() -> (Document, Page, f64, f64, f64) {
    let mut doc = Document::new();
    doc.set_title("Facture");
    let page = Page::a4();

    let page_height = mm(297.0);
    let margin_l = mm(20.0);
    let margin_r = mm(190.0);
    let y_position = page_height - mm(18.0);

    (doc, page, margin_l, margin_r, y_position)
  }
}
