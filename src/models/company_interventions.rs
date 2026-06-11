use chrono::NaiveDate;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait};

use crate::models::{
  _entities::company_interventions,
  my_errors::{application_error::ApplicationError, MyErrors},
};

pub struct InterventionParams {
  pub quantity: i32,
  pub unit_price: f32,
  pub vat_rate: Decimal,
  pub issue_date: NaiveDate,
  pub object: String,
}

const ALLOWED_VAT_VALUES: [f32; 4] = [0.0, 5.5, 10.0, 20.0];

impl company_interventions::ActiveModel {
  pub async fn create<T: ConnectionTrait>(
    db: &T,
    practitioner_id: i32,
    company_id: i32,
    params: &InterventionParams,
  ) -> Result<company_interventions::Model, MyErrors> {
    validate_vat_values(&params.vat_rate)?;

    let unit_price_in_cents = (params.unit_price * 100.0)
      .round()
      .to_i32()
      .ok_or(ApplicationError::UnprocessableEntity)?;

    Ok(
      Self {
        company_id: ActiveValue::Set(company_id),
        practitioner_id: ActiveValue::Set(practitioner_id),
        quantity: ActiveValue::Set(params.quantity),
        unit_price_in_cents: ActiveValue::Set(unit_price_in_cents),
        vat_rate_in_percent: ActiveValue::Set(params.vat_rate),
        issue_date: ActiveValue::Set(params.issue_date),
        object: ActiveValue::Set(params.object.clone()),
        ..Default::default()
      }
      .insert(db)
      .await?,
    )
  }

  pub async fn update<T: ConnectionTrait>(
    mut self,
    db: &T,
    params: &InterventionParams,
  ) -> Result<(), MyErrors> {
    validate_vat_values(&params.vat_rate)?;

    let unit_price_in_cents = (params.unit_price * 100.0)
      .round()
      .to_i32()
      .ok_or(ApplicationError::UnprocessableEntity)?;

    self.quantity = ActiveValue::Set(params.quantity);
    self.unit_price_in_cents = ActiveValue::Set(unit_price_in_cents);
    self.vat_rate_in_percent = ActiveValue::Set(params.vat_rate);
    self.issue_date = ActiveValue::Set(params.issue_date);
    self.object = ActiveValue::Set(params.object.clone());

    self.save(db).await?;

    Ok(())
  }
}

fn validate_vat_values(vat_rate: &Decimal) -> Result<(), MyErrors> {
  let vat_rate = vat_rate
    .to_f32()
    .ok_or(ApplicationError::UnprocessableEntity)?;

  if !ALLOWED_VAT_VALUES.contains(&vat_rate) {
    return Err(ApplicationError::UnprocessableEntity.into());
  }

  Ok(())
}
