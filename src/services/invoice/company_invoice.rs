use sea_orm::DatabaseConnection;

use crate::{
  models::{_entities::company_interventions, my_errors::MyErrors, practitioner_offices, users},
  services::{
    invoice::{
      pdf::company::{CompanyInvoiceGenerator, CompanyPdfArgs},
      Invoice, InvoiceKind,
    },
    storage::StorageService,
  },
};

pub async fn generate(
  company_intervention: &company_interventions::Model,
  current_user: &users::Model,
  practitioner_office: practitioner_offices::Model,
  db: &DatabaseConnection,
) -> Result<Invoice, MyErrors> {
  let business_info = current_user.business_information(db).await?;
  let company = company_intervention.company(db).await?;

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

  let filename = format!(
    "{} Facture {} {}.pdf",
    current_user.full_name(),
    company.name,
    company_intervention.issue_date.format("%d_%m_%Y"),
  );

  let args = CompanyPdfArgs {
    intervention: company_intervention.clone(),
    user: current_user.clone(),
    business_info,
    company,
    emission_date,
    practitioner_office,
    signature_data,
  };

  let data = CompanyInvoiceGenerator::new(args).build()?.to_bytes()?;

  Ok(Invoice {
    data,
    filename,
    date: company_intervention.issue_date,
    kind: InvoiceKind::Company,
  })
}
