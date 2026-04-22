use crate::models::my_errors::MyErrors;
use axum::http::StatusCode;

#[derive(Debug)]
pub enum UnexpectedError {
  ShouldNotHappen,
  #[allow(non_camel_case_types)]
  new(String),
}

impl From<UnexpectedError> for MyErrors {
  fn from(err: UnexpectedError) -> Self {
    match err {
      UnexpectedError::ShouldNotHappen => MyErrors {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg: "should_not_happen".to_string(),
      },
      UnexpectedError::new(msg) => MyErrors {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        msg,
      },
    }
  }
}
