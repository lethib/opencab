use crate::{
  app_state::AppState,
  db::DB,
  auth::{
    jwt::{JwtService, TOKEN_TYPE_AUTH},
    statement::AuthStatement,
  },
  models::{
    _entities::user_business_informations,
    my_errors::{
      authentication_error::AuthenticationError, unexpected_error::UnexpectedError, MyErrors,
    },
    users,
  },
};

pub struct AuthContext {
  pub current_user: Option<(users::Model, Option<user_business_informations::Model>)>,
  authorized: bool,
  complete: bool,
  pub error: Option<MyErrors>,
}

impl AuthContext {
  pub async fn new(auth_header: Option<&str>, state: &AppState) -> Self {
    let (current_user, error) = match auth_header {
      Some(header) => Self::validate_auth_header(header, state).await,
      None => (None, None),
    };

    Self {
      current_user,
      authorized: false,
      complete: false,
      error: error.map(|e| e.into()),
    }
  }

  pub fn authorize(self) -> AuthStatement {
    AuthStatement::new(self)
  }

  pub(super) fn authorized(&mut self) -> Result<(), MyErrors> {
    self.ensure_not_completed()?;
    self.authorized = true;
    Ok(())
  }

  pub(super) fn not_authorized(&mut self, error: Option<MyErrors>) -> Result<(), MyErrors> {
    self.ensure_not_completed()?;
    self.authorized = false;

    if self.error.is_none() {
      self.error = error;
    }

    Ok(())
  }

  pub(super) fn complete(&mut self) -> Result<(), MyErrors> {
    self.ensure_not_completed()?;
    self.complete = true;

    if !self.authorized {
      match self.error.take() {
        Some(error) => return Err(error),
        None => return Err(AuthenticationError::AccessDenied(None).into()),
      }
    }

    Ok(())
  }

  pub(super) async fn validate_auth_header(
    auth_header: &str,
    state: &AppState,
  ) -> (
    Option<(users::Model, Option<user_business_informations::Model>)>,
    Option<AuthenticationError>,
  ) {
    let token = match auth_header.strip_prefix("Bearer ") {
      Some(t) => t,
      None => return (None, Some(AuthenticationError::MissingToken)),
    };

    let jwt_service = JwtService::new(&state.config.jwt.secret);
    let claims = match jwt_service.validate_token(token) {
      Ok(data) => data,
      Err(_) => return (None, Some(AuthenticationError::InvalidToken)),
    };

    if claims.token_type != TOKEN_TYPE_AUTH {
      return (None, Some(AuthenticationError::InvalidToken));
    }

    let user_result = match users::Model::find_by_pid(DB::get(), &claims.pid).await {
      Ok(user) => user,
      Err(_) => return (None, Some(AuthenticationError::InvalidClaims)),
    };

    if !user_result.0.is_access_key_verified {
      return (None, Some(AuthenticationError::AccessKeyNotVerified));
    }

    (Some(user_result), None)
  }

  fn ensure_not_completed(&self) -> Result<(), MyErrors> {
    if self.complete {
      return Err(UnexpectedError::ShouldNotHappen.into());
    }
    Ok(())
  }
}
