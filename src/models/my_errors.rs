pub mod application_error;
pub mod authentication_error;
pub mod unexpected_error;

use axum::response::IntoResponse;
use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, PartialEq)]
pub struct MyErrors {
  pub code: axum::http::StatusCode,
  pub msg: String,
}

impl Serialize for MyErrors {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut my_errors = serializer.serialize_struct("MyErrors", 2)?;
    my_errors.serialize_field("code", &self.code.as_u16())?;
    my_errors.serialize_field("msg", &self.msg)?;
    my_errors.end()
  }
}

impl std::fmt::Display for MyErrors {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl IntoResponse for MyErrors {
  fn into_response(self) -> axum::response::Response {
    tracing::warn!("Error message: {}", self.msg);
    (self.code, axum::Json(self)).into_response()
  }
}

impl<T> From<T> for MyErrors
where
  T: std::error::Error,
{
  fn from(err: T) -> Self {
    MyErrors {
      code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
      msg: err.to_string(),
    }
  }
}
