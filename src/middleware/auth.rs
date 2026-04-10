use axum::{
  extract::{FromRequestParts, Request},
  http::request::Parts,
  middleware::Next,
  response::Response,
};

use crate::{
  auth::{context::AuthContext, statement::AuthStatement},
  models::{
    _entities::user_business_informations,
    my_errors::{authentication_error::AuthenticationError, MyErrors},
    users,
  },
};

/// Extracteur pour les routes qui nécessitent un utilisateur authentifié.
/// Retourne une erreur 401 si l'utilisateur n'est pas authentifié.
pub struct AuthenticatedUser(
  pub users::Model,
  pub Option<user_business_informations::Model>,
);

pub async fn authenticated_request(
  authorize: AuthStatement,
  request: Request,
  next: Next,
) -> Result<Response, MyErrors> {
  authorize.authenticated_user().run_complete()?;

  Ok(next.run(request).await)
}

impl<S> FromRequestParts<S> for AuthStatement
where
  S: Send + Sync,
{
  type Rejection = MyErrors;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let auth_header = parts
      .headers
      .get("Authorization")
      .and_then(|h| h.to_str().ok());

    Ok(AuthContext::new(auth_header).await.authorize())
  }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
  S: Send + Sync,
{
  type Rejection = MyErrors;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let auth_headers = parts
      .headers
      .get("Authorization")
      .and_then(|h| h.to_str().ok());

    let auth_context = AuthContext::new(auth_headers).await;

    match auth_context.current_user {
      Some(user) => Ok(AuthenticatedUser(user.0, user.1)),
      None => Err(
        auth_context
          .error
          .unwrap_or(AuthenticationError::InvalidClaims.into()),
      ),
    }
  }
}
