use chrono::NaiveDate;
use oxidize_pdf::{Color, Document, Font, Page};

use crate::{
  models::{
    _entities::user_business_informations,
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
    patients, practitioner_offices,
    users::users,
  },
  services::invoice::pdf::{embed_signature_image, mm},
};

pub(in crate::services::invoice) struct PatientPdfArgs {
  pub user: users::Model,
  pub business_info: user_business_informations::Model,
  pub patient: patients::Model,
  pub decrypted_patient_ssn: Option<String>,
  pub amount: f32,
  pub date: NaiveDate,
  pub office: practitioner_offices::Model,
  pub signature_data: Option<Vec<u8>>,
}

pub(in crate::services::invoice) struct PatientInvoiceGenerator {
  pub args: PatientPdfArgs,
  doc: Document,
  page: Page,
  y_position: f64,
  margin: f64,
}

impl PatientInvoiceGenerator {
  pub(in crate::services::invoice) fn new(args: PatientPdfArgs) -> Self {
    let (doc, page, margin, y_position) = Self::setup_document();

    Self {
      args,
      doc,
      page,
      margin,
      y_position,
    }
  }

  pub(in crate::services::invoice) fn build(mut self) -> Result<Self, MyErrors> {
    self.build_header()?;
    self.y_position -= mm(30.0);

    // === INVOICE TITLE - CENTERED ===
    let title = "Note d'honoraires acquittée";
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 20.0)
      .at(mm(60.0), self.y_position)
      .write(title)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(30.0);

    self.build_patient_information()?;
    self.y_position -= mm(18.0);

    self.build_amount()?;
    self.y_position -= mm(35.0);

    self.build_footer()?;

    Ok(self)
  }

  fn build_header(&mut self) -> Result<(), MyErrors> {
    let full_name = format!(
      "{} – {}",
      &self.args.user.full_name(),
      &self.args.business_info.profession.to_french()
    );
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 14.0)
      .at(self.margin, self.y_position)
      .write(&full_name)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(12.0);

    // Professional numbers with consistent formatting
    if let Some(ref adeli) = self.args.business_info.adeli_number {
      self
        .page
        .text()
        .set_font(Font::Helvetica, 10.0)
        .at(self.margin, self.y_position)
        .write(&format!("N° Adeli : {}", adeli))
        .map_err(|e| UnexpectedError::new(e.to_string()))?;
      self.y_position -= mm(5.0);
    }

    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&format!("N°RPPS : {}", self.args.business_info.rpps_number))
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(5.0);

    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&format!(
        "N°SIRET : {}",
        self.args.business_info.siret_number
      ))
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(12.0);

    // Address
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&self.args.office.address_line_1)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(5.0);

    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&format!(
        "{} {}",
        self.args.office.address_zip_code, self.args.office.address_city,
      ))
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(8.0);

    // Contact info
    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&format!("Tel : {}", &self.args.user.phone_number))
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    self.y_position -= mm(8.0);

    self
      .page
      .text()
      .set_font(Font::Helvetica, 10.0)
      .at(self.margin, self.y_position)
      .write(&self.args.user.email)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    Ok(())
  }

  fn build_patient_information(&mut self) -> Result<(), MyErrors> {
    let patient_full_name = format!(
      "{} {}",
      self.args.patient.last_name, self.args.patient.first_name
    );
    let full_text = format!("Reçu de : {}", patient_full_name);

    self
      .page
      .text()
      .set_font(Font::Helvetica, 11.0)
      .at(self.margin, self.y_position)
      .write(&full_text)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;

    // Draw underline only for "Reçu de :"
    let underline_y = self.y_position - mm(1.0);
    self
      .page
      .graphics()
      .set_stroke_color(Color::black())
      .set_line_width(mm(0.3))
      .move_to(self.margin, underline_y)
      .line_to(self.margin + mm(17.0), underline_y)
      .stroke();

    self.y_position -= mm(12.0);

    if let Some(ref patient_ssn) = self.args.decrypted_patient_ssn {
      // Social security number with box
      let ssn_y = self.y_position;
      self
        .page
        .text()
        .set_font(Font::Helvetica, 11.0)
        .at(self.margin, self.y_position)
        .write(&format!("Numéro de sécurité sociale : {}", patient_ssn))
        .map_err(|e| UnexpectedError::new(e.to_string()))?;

      // Draw box around SSN field
      let box_x = self.margin - mm(2.0);
      let box_y = ssn_y - mm(3.0);
      let box_width = mm(185.0) - box_x;
      let box_height = mm(8.0);

      self
        .page
        .graphics()
        .set_stroke_color(Color::black())
        .set_line_width(mm(0.5))
        .rect(box_x, box_y, box_width, box_height)
        .stroke();

      self.y_position -= mm(18.0);
    }

    // Address with box
    let addr_y = self.y_position;
    let address_text = format!(
      "Adresse : {} – {} {}",
      self.args.patient.address_line_1,
      self.args.patient.address_zip_code,
      self.args.patient.address_city
    );
    self
      .page
      .text()
      .set_font(Font::Helvetica, 11.0)
      .at(self.margin, self.y_position)
      .write(&address_text)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;

    // Draw box around address field
    let addr_box_x = self.margin - mm(2.0);
    let addr_box_y = addr_y - mm(3.0);
    let addr_box_width = mm(185.0) - addr_box_x;
    let addr_box_height = mm(8.0);

    self
      .page
      .graphics()
      .set_stroke_color(Color::black())
      .set_line_width(mm(0.5))
      .rect(addr_box_x, addr_box_y, addr_box_width, addr_box_height)
      .stroke();
    Ok(())
  }

  fn build_amount(&mut self) -> Result<(), MyErrors> {
    let full_text = format!("Honoraire : {:.2}€", self.args.amount);

    self
      .page
      .text()
      .set_font(Font::Helvetica, 11.0)
      .at(self.margin, self.y_position)
      .write(&full_text)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;

    // Draw underline only for "Honoraire :"
    let underline_text = "Honoraire :";
    let text_width = underline_text.len() as f64 * 2.5;
    let underline_y = self.y_position - mm(1.0);

    self
      .page
      .graphics()
      .set_stroke_color(Color::black())
      .set_line_width(mm(0.3))
      .move_to(self.margin, underline_y)
      .line_to(self.margin + mm(text_width), underline_y)
      .stroke();
    Ok(())
  }

  fn build_footer(&mut self) -> Result<(), MyErrors> {
    let invoice_date_str = self.args.date.format("%d/%m/%Y").to_string();
    let date_location = format!(
      "Fait à {}, le {}",
      self.args.office.address_city, invoice_date_str
    );

    // Right align date
    let date_x = mm(230.0) - self.margin - mm(85.0);
    self
      .page
      .text()
      .set_font(Font::Helvetica, 11.0)
      .at(date_x, self.y_position)
      .write(&date_location)
      .map_err(|e| UnexpectedError::new(e.to_string()))?;

    // Try to embed signature image if available
    if let Some(sig_bytes) = self.args.signature_data.take() {
      match embed_signature_image(&mut self.page, sig_bytes, mm(30.0), self.y_position) {
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

    self.y_position -= mm(30.0);

    // Practitioner name below signature
    self
      .page
      .text()
      .set_font(Font::Helvetica, 11.0)
      .at(date_x + mm(20.0), self.y_position)
      .write(&self.args.user.full_name())
      .map_err(|e| UnexpectedError::new(e.to_string()))?;
    Ok(())
  }

  pub(in crate::services::invoice) fn with_duplicata(mut self) -> Result<Self, MyErrors> {
    self
      .page
      .text()
      .set_font(Font::HelveticaBold, 70.0)
      .set_fill_color(Color::gray(0.82))
      .set_character_spacing(mm(2.5))
      .at(mm(22.0), mm(148.5))
      .write("DUPLICATA")
      .map_err(|e| UnexpectedError::new(e.to_string()))?;

    Ok(self)
  }

  pub(in crate::services::invoice) fn to_bytes(mut self) -> Result<Vec<u8>, MyErrors> {
    self.doc.add_page(self.page);
    self
      .doc
      .to_bytes()
      .map_err(|e| UnexpectedError::new(e.to_string()).into())
  }

  fn setup_document() -> (Document, Page, f64, f64) {
    let mut document = Document::new();
    document.set_title("Note d'honoraires acquitée");

    let page = Page::a4();

    let page_height = mm(297.0);
    let margin = mm(25.0);
    let y_position = page_height - margin - mm(10.0);

    (document, page, margin, y_position)
  }
}
