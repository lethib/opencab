use axum::extract::FromRequestParts;
use sea_orm::DatabaseConnection;

use crate::{
  auth::{context::AuthContext, statement::AuthStatement},
  config::Config,
  models::{
    my_errors::{authentication_error::AuthenticationError, MyErrors},
    users::users,
  },
};

#[derive(Clone)]
pub struct AppState {
  pub db: DatabaseConnection,
  pub config: Config,
}

pub struct Ctx {
  pub db: DatabaseConnection,
  pub current_user: users::Model,
}

impl FromRequestParts<AppState> for Ctx {
  type Rejection = MyErrors;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let header = parts
      .headers
      .get("Authorization")
      .and_then(|h| h.to_str().ok());

    let (user, error) = match header {
      Some(h) => AuthContext::validate_auth_header(h, &state.db, &state.config.jwt.secret).await,
      None => (None, Some(AuthenticationError::MissingToken)),
    };

    let current_user = user
      .ok_or_else(|| error.unwrap_or(AuthenticationError::InvalidClaims))?
      .0;

    Ok(Ctx {
      db: state.db.clone(),
      current_user,
    })
  }
}

impl Ctx {
  pub fn authorize<'statement>(&'statement self) -> AuthStatement<'statement, 'statement> {
    AuthContext::for_user(&self.current_user).authorize(&self.db)
  }
}
