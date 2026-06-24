use std::error::Error;

use crate::models::my_errors::{unexpected_error::Kind::Custom, MyErrors};
use axum::http::StatusCode;

pub struct UnexpectedError {
  kind: Kind,
}

#[derive(Debug)]
pub enum Kind {
  ShouldNotHappen,
  Custom(Box<dyn Error + Send + Sync>),
}

impl UnexpectedError {
  pub fn should_not_happen() -> Self {
    UnexpectedError {
      kind: Kind::ShouldNotHappen,
    }
  }

  pub fn new(err: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
    UnexpectedError {
      kind: Custom(err.into()),
    }
  }
}

impl From<UnexpectedError> for MyErrors {
  fn from(err: UnexpectedError) -> Self {
    match err.kind {
      Kind::ShouldNotHappen => MyErrors {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: "should_not_happen".to_string(),
      },
      Kind::Custom(error) => MyErrors {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: error.to_string(),
      },
    }
  }
}
