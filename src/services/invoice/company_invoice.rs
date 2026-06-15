use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
  db::DB,
  models::{
    _entities::{company_interventions, practitioner_companies, user_business_informations},
    my_errors::{application_error::ApplicationError, MyErrors},
    practitioner_offices, users,
  },
  services::{
    invoice::pdf::company::{CompanyInvoiceGenerator, CompanyPdfArgs},
    storage::StorageService,
  },
};

pub async fn generate(
  company_intervention: &company_interventions::Model,
  current_user: &users::Model,
  practitioner_office: practitioner_offices::Model,
) -> Result<Vec<u8>, MyErrors> {
  let business_info = user_business_informations::Entity::find()
    .filter(user_business_informations::Column::UserId.eq(current_user.id))
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let company = practitioner_companies::Entity::find_by_id(company_intervention.company_id)
    .one(DB::get())
    .await?
    .ok_or(ApplicationError::NotFound)?;

  let emission_date = chrono::Utc::now().date_naive();

  let signature_data = match StorageService::new() {
    Ok(service) => {
      if let Some(ref sig_name) = business_info.signature_file_name {
        match service.fetch_signature(sig_name).await {
          Ok(data) => Some(data),
          Err(e) => {
            tracing::warn!(
              "Failed to fetch signature for company invoice: {}. Continuing without.",
              e
            );
            None
          }
        }
      } else {
        None
      }
    }
    Err(e) => {
      tracing::warn!(
        "Storage service unavailable for company invoice: {}. Continuing without signature.",
        e
      );
      None
    }
  };

  let args = CompanyPdfArgs {
    intervention: company_intervention.clone(),
    user: current_user.clone(),
    business_info,
    company,
    emission_date,
    practitioner_office,
    signature_data,
  };

  CompanyInvoiceGenerator::new(args).build()?.to_bytes()
}
