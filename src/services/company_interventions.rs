use chrono::NaiveDate;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{IntoActiveModel, TransactionTrait, TryIntoModel};

use crate::models::{
  _entities::company_interventions::{self, ActiveModelEx, Model as CompanyIntervention},
  company_interventions::ALLOWED_VAT_VALUES,
  my_errors::{application_error::ApplicationError, unexpected_error::UnexpectedError, MyErrors},
};

pub struct CompanyInterventionsService {
  pub intervention: CompanyIntervention,
}

impl CompanyInterventionsService {
  pub fn for_intervention(intervention: CompanyIntervention) -> Self {
    Self { intervention }
  }

  pub fn create(params: InterventionParams) -> Result<ActiveModelEx, MyErrors> {
    validate_vat_values(&params.vat_rate)?;

    let unit_price_in_cents = (params.unit_price * 100.0)
      .round()
      .to_i32()
      .ok_or(UnexpectedError::should_not_happen())?;

    Ok(
      company_interventions::ActiveModel::builder()
        .set_quantity(params.quantity)
        .set_unit_price_in_cents(unit_price_in_cents)
        .set_vat_rate_in_percent(params.vat_rate)
        .set_issue_date(params.issue_date)
        .set_object(params.object),
    )
  }

  pub fn update(self, params: InterventionParams) -> Result<ActiveModelEx, MyErrors> {
    validate_vat_values(&params.vat_rate)?;

    let unit_price_in_cents = (params.unit_price * 100.0)
      .round()
      .to_i32()
      .ok_or(UnexpectedError::should_not_happen())?;

    let intervention = self.intervention.into_active_model().into_ex();

    Ok(
      intervention
        .set_quantity(params.quantity)
        .set_unit_price_in_cents(unit_price_in_cents)
        .set_vat_rate_in_percent(params.vat_rate)
        .set_issue_date(params.issue_date)
        .set_object(params.object),
    )
  }
}

impl ActiveModelEx {
  pub fn for_company(self, company_id: i32) -> Self {
    self.set_company_id(company_id)
  }

  pub fn for_practitioner(self, practitioner_id: i32) -> Self {
    self.set_practitioner_id(practitioner_id)
  }

  pub async fn build(self, db: &impl TransactionTrait) -> Result<CompanyIntervention, MyErrors> {
    Ok(self.save(db).await?.try_into_model()?.into())
  }
}

pub struct InterventionParams {
  pub quantity: i32,
  pub unit_price: f32,
  pub vat_rate: Decimal,
  pub issue_date: NaiveDate,
  pub object: String,
}

fn validate_vat_values(vat_rate: &Decimal) -> Result<(), MyErrors> {
  let vat_rate = vat_rate.to_f32().ok_or(UnexpectedError::should_not_happen())?;

  if !ALLOWED_VAT_VALUES.contains(&vat_rate) {
    return Err(ApplicationError::unprocessable_entity("invalid_vat_values").into());
  }

  Ok(())
}
