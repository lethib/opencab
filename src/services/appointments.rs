use chrono::{Datelike, NaiveDate};
use rust_xlsxwriter::*;
use sea_orm::{ActiveEnum, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

use crate::models::{
  _entities::{medical_appointments, patients, practitioner_offices, user_practitioner_offices},
  my_errors::{unexpected_error::UnexpectedError, MyErrors},
  users,
};

pub struct MedicalAppointmentExtractor<'user> {
  user: &'user users::Model,
}

#[derive(Debug)]
pub struct MedicalAppointmentDetail {
  pub appointment: medical_appointments::Model,
  pub patient: patients::Model,
  pub office: practitioner_offices::Model,
  pub revenue_share_percentage: f64,
}

trait MonthToString {
  fn to_french(&self) -> Result<&'static str, MyErrors>;
}

impl MonthToString for u32 {
  fn to_french(&self) -> Result<&'static str, MyErrors> {
    let french_translation = match self {
      1 => "Janvier",
      2 => "Février",
      3 => "Mars",
      4 => "Avril",
      5 => "Mai",
      6 => "Juin",
      7 => "Juillet",
      8 => "Août",
      9 => "Septembre",
      10 => "Octobre",
      11 => "Novembre",
      12 => "Décembre",
      _ => return Err(UnexpectedError::new("number_outside_months_range".to_string()).into()),
    };

    Ok(french_translation)
  }
}

fn write_headers(
  worksheet: &mut Worksheet,
  headers: &[(&str, u16)],
  format: &Format,
) -> Result<(), MyErrors> {
  for (col, (label, width)) in headers.iter().enumerate() {
    worksheet.write_with_format(0, col as u16, *label, format)?;
    worksheet.set_column_width(col as u16, *width)?;
  }
  Ok(())
}

pub trait ToExcel {
  fn export_appointments(&self) -> Result<Workbook, MyErrors>;
  fn generate_accountability(&self) -> Result<Workbook, MyErrors>;
}

impl ToExcel for Vec<MedicalAppointmentDetail> {
  fn generate_accountability(&self) -> Result<Workbook, MyErrors> {
    let mut appointments_per_month: HashMap<u32, Vec<&MedicalAppointmentDetail>> = HashMap::new();

    for detail in self {
      appointments_per_month
        .entry(detail.appointment.date.month())
        .or_default()
        .push(detail);
    }

    // First pass: compute annual totals (accumulate in cents to avoid f64 drift)
    let mut annual_revenue_cents: i64 = 0;
    let mut annual_hand_back_cents: i64 = 0;

    for detail in self {
      let price_cents = detail.appointment.price_in_cents as i64;
      let hand_back_cents =
        (price_cents as f64 * detail.revenue_share_percentage / 100.0).round() as i64;
      annual_revenue_cents += price_cents;
      annual_hand_back_cents += hand_back_cents;
    }

    let annual_revenue = annual_revenue_cents as f64 / 100.0;
    let annual_hand_back = annual_hand_back_cents as f64 / 100.0;
    let annual_user_revenue = (annual_revenue_cents - annual_hand_back_cents) as f64 / 100.0;

    let mut workbook = Workbook::new();
    let revenue_format = Format::new().set_num_format("0.00");
    let header_format = Format::new()
      .set_bold()
      .set_background_color(Color::Green)
      .set_font_color(Color::White);
    let date_format = Format::new().set_num_format("dd/mm/yyyy");

    // Dashboard sheet (first)
    let dashboard = workbook.add_worksheet().set_name("Récapitulatif")?;
    dashboard.set_column_width(0, 30)?;
    dashboard.set_column_width(1, 20)?;
    dashboard.write_with_format(0, 0, "Indicateur", &header_format)?;
    dashboard.write_with_format(0, 1, "Montant (€)", &header_format)?;
    dashboard.write(1, 0, "CA total")?;
    dashboard.write_with_format(1, 1, annual_revenue, &revenue_format)?;
    dashboard.write(2, 0, "Votre CA")?;
    dashboard.write_with_format(2, 1, annual_user_revenue, &revenue_format)?;
    dashboard.write(3, 0, "Rétrocession totale")?;
    dashboard.write_with_format(3, 1, annual_hand_back, &revenue_format)?;

    // Monthly sheets
    let mut sorted_monthly_appointments: Vec<_> = appointments_per_month.iter().collect();
    sorted_monthly_appointments.sort_by_key(|(month, _)| **month);

    for (month, monthly_appointments) in sorted_monthly_appointments {
      let worksheet = workbook.add_worksheet().set_name(month.to_french()?)?;

      write_headers(
        worksheet,
        &[
          ("Date", 15),
          ("Nom", 20),
          ("Prénom", 20),
          ("Mode de paiement", 20),
          ("Cabinet", 20),
          ("Prix consultation (€)", 20),
          ("Votre CA (€)", 20),
          ("Rétrocession (€)", 20),
        ],
        &header_format,
      )?;

      let mut monthly_revenue: f64 = 0.0;
      let mut monthly_user_revenue: f64 = 0.0;
      let mut monthly_hand_back: f64 = 0.0;

      for (i, detail) in monthly_appointments.iter().enumerate() {
        let row = i as u32 + 1;
        let excel_date = ExcelDateTime::parse_from_str(&detail.appointment.date.to_string())?;
        let price = detail.appointment.price_in_cents as f64 / 100.0;
        let hand_back = price * detail.revenue_share_percentage / 100.0;

        monthly_revenue += price;
        monthly_user_revenue += price - hand_back;
        monthly_hand_back += hand_back;

        worksheet.write_with_format(row, 0, &excel_date, &date_format)?;
        worksheet.write(row, 1, &detail.patient.last_name)?;
        worksheet.write(row, 2, &detail.patient.first_name)?;
        worksheet.write(
          row,
          3,
          detail
            .appointment
            .payment_method
            .as_ref()
            .map(|p| p.to_french()),
        )?;
        worksheet.write(row, 4, detail.office.name.clone())?;
        worksheet.write(row, 5, price)?;
        worksheet.write_with_format(row, 6, price - hand_back, &revenue_format)?;
        worksheet.write_with_format(row, 7, hand_back, &revenue_format)?;
      }

      let total_format = Format::new()
        .set_background_color(Color::Gray)
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(12)
        .set_bold();
      let total_row = monthly_appointments.len() as u32 + 1;

      worksheet.merge_range(total_row, 0, total_row, 4, "TOTAL", &total_format)?;
      worksheet.write_with_format(total_row, 5, monthly_revenue, &revenue_format)?;
      worksheet.write_with_format(total_row, 6, monthly_user_revenue, &revenue_format)?;
      worksheet.write_with_format(total_row, 7, monthly_hand_back, &revenue_format)?;
    }

    Ok(workbook)
  }

  fn export_appointments(&self) -> Result<Workbook, MyErrors> {
    let mut appointments_by_office: HashMap<String, Vec<&MedicalAppointmentDetail>> =
      HashMap::new();

    for detail in self {
      appointments_by_office
        .entry(detail.office.name.clone())
        .or_default()
        .push(detail);
    }

    let mut workbook = Workbook::new();
    let date_format = Format::new().set_num_format("dd/mm/yyyy");
    let revenue_format = Format::new().set_num_format("0.00");
    let header_format = Format::new()
      .set_bold()
      .set_background_color(Color::Green)
      .set_font_color(Color::White);

    let mut sorted_offices: Vec<_> = appointments_by_office.iter().collect();
    sorted_offices.sort_by_key(|(name, _)| name.as_str());

    for (office_name, office_appointments) in sorted_offices {
      let worksheet = workbook.add_worksheet();
      worksheet.set_name(office_name)?;

      write_headers(
        worksheet,
        &[
          ("Date", 15),
          ("Nom", 20),
          ("Prénom", 20),
          ("Mode de paiement", 15),
          ("Prix consultation (€)", 20),
          ("Votre CA (€)", 20),
          ("Rétrocession (€)", 20),
        ],
        &header_format,
      )?;

      for (i, detail) in office_appointments.iter().enumerate() {
        let row = i as u32 + 1;
        let excel_date = ExcelDateTime::parse_from_str(&detail.appointment.date.to_string())?;
        let price = detail.appointment.price_in_cents as f64 / 100.0;
        let hand_back = price * detail.revenue_share_percentage / 100.0;

        worksheet.write_with_format(row, 0, &excel_date, &date_format)?;
        worksheet.write(row, 1, &detail.patient.last_name)?;
        worksheet.write(row, 2, &detail.patient.first_name)?;
        worksheet.write(
          row,
          3,
          detail
            .appointment
            .payment_method
            .clone()
            .map(|p| p.to_value()),
        )?;
        worksheet.write(row, 4, price)?;
        worksheet.write_with_format(row, 5, price - hand_back, &revenue_format)?;
        worksheet.write_with_format(row, 6, hand_back, &revenue_format)?;
      }
    }

    Ok(workbook)
  }
}

impl<'user> MedicalAppointmentExtractor<'user> {
  pub fn for_user(user: &'user users::Model) -> Self {
    MedicalAppointmentExtractor { user }
  }

  pub async fn extract(
    &self,
    db: &DatabaseConnection,
    start_date: NaiveDate,
    end_date: NaiveDate,
  ) -> Result<Vec<MedicalAppointmentDetail>, MyErrors> {
    let appointments = medical_appointments::Entity::find()
      .filter(medical_appointments::Column::UserId.eq(self.user.id))
      .filter(medical_appointments::Column::Date.between(start_date, end_date))
      .inner_join(patients::Entity)
      .inner_join(practitioner_offices::Entity)
      .select_also(patients::Entity)
      .select_also(practitioner_offices::Entity)
      .order_by_asc(medical_appointments::Column::Date)
      .order_by_asc(patients::Column::LastName)
      .all(db)
      .await?;

    let user_offices = user_practitioner_offices::Entity::find()
      .filter(user_practitioner_offices::Column::UserId.eq(self.user.id))
      .all(db)
      .await?;

    let revenue_share_by_office: HashMap<i32, f64> = user_offices
      .into_iter()
      .map(|uo| {
        let pct = uo.revenue_share_percentage.try_into().unwrap_or(0.0);
        (uo.practitioner_office_id, pct)
      })
      .collect();

    let results = appointments
      .into_iter()
      .map(|(appointment, patient, office)| -> Result<_, MyErrors> {
        let office = office.ok_or(UnexpectedError::new("office_should_be_defined".to_string()))?;
        let revenue_share_percentage =
          *revenue_share_by_office
            .get(&office.id)
            .ok_or(UnexpectedError::new(
              "revenue_share_percentage_should_be_defined".to_string(),
            ))?;
        Ok(MedicalAppointmentDetail {
          appointment,
          patient: patient.ok_or(UnexpectedError::new(
            "patient_should_be_defined".to_string(),
          ))?,
          office,
          revenue_share_percentage,
        })
      })
      .collect::<Result<Vec<_>, MyErrors>>()?;

    Ok(results)
  }
}
