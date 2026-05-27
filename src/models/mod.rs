pub mod _entities;
pub mod company_interventions;
pub mod enums;
pub mod medical_appointments;
pub mod my_errors;
pub mod patients;
pub mod practitioner_companies;
pub mod practitioner_offices;
pub mod user_business_informations;
pub mod user_practitioner_offices;
pub mod users;

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug)]
pub enum ModelError {
  EntityNotFound,
  EntityAlreadyExists,
  Any(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for ModelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModelError::EntityNotFound => write!(f, "Entity not found"),
      ModelError::EntityAlreadyExists => write!(f, "Entity already exists"),
      ModelError::Any(e) => write!(f, "Model error: {}", e),
    }
  }
}

impl std::error::Error for ModelError {}

impl From<sea_orm::DbErr> for ModelError {
  fn from(err: sea_orm::DbErr) -> Self {
    ModelError::Any(Box::new(err))
  }
}
