use std::error::Error;

use crate::models::my_errors::MyErrors;
use axum::http::StatusCode;

pub struct ApplicationError {
  kind: Kind,
}

enum Kind {
  UnprocessableEntity(Box<dyn Error + Send + Sync>),
  NotFound,
  BadRequest(Box<dyn Error + Send + Sync>),
}

impl ApplicationError {
  pub fn unprocessable_entity(err: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
    ApplicationError {
      kind: Kind::UnprocessableEntity(err.into()),
    }
  }

  pub fn bad_request(err: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
    ApplicationError {
      kind: Kind::BadRequest(err.into()),
    }
  }

  pub fn not_found() -> Self {
    ApplicationError { kind: Kind::NotFound }
  }
}

impl From<ApplicationError> for MyErrors {
  fn from(err: ApplicationError) -> Self {
    match err.kind {
      Kind::UnprocessableEntity(source) => MyErrors {
        code: StatusCode::UNPROCESSABLE_ENTITY,
        msg: source.to_string(),
      },
      Kind::NotFound => MyErrors {
        code: StatusCode::NOT_FOUND,
        msg: "resource_not_found".into(),
      },
      Kind::BadRequest(msg) => MyErrors {
        code: StatusCode::BAD_REQUEST,
        msg: msg.to_string(),
      },
    }
  }
}
