use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::{
  auth::{
    jwt::{JwtService, TOKEN_TYPE_AUTH},
    statement::AuthStatement,
  },
  models::{
    my_errors::{authentication_error::AuthenticationError, unexpected_error::UnexpectedError, MyErrors},
    users,
  },
};

pub struct AuthContext<'user> {
  pub current_user: &'user users::Model,
  authorized: bool,
  complete: bool,
  pub error: Option<MyErrors>,
}

impl<'user> AuthContext<'user> {
  pub fn for_user(user: &'user users::Model) -> Self {
    Self {
      current_user: user,
      authorized: false,
      complete: false,
      error: None,
    }
  }

  pub fn authorize<'db>(self, db: &'db DatabaseConnection) -> AuthStatement<'user, 'db> {
    AuthStatement::new(self, db)
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

  pub async fn validate_auth_header(
    auth_header: &str,
    db: &impl ConnectionTrait,
    jwt_secret: &str,
  ) -> (Option<users::Model>, Option<AuthenticationError>) {
    let token = match auth_header.strip_prefix("Bearer ") {
      Some(t) => t,
      None => return (None, Some(AuthenticationError::MissingToken)),
    };

    let jwt_service = JwtService::new(jwt_secret);
    let claims = match jwt_service.validate_token(token) {
      Ok(data) => data,
      Err(_) => return (None, Some(AuthenticationError::InvalidToken)),
    };

    if claims.token_type != TOKEN_TYPE_AUTH {
      return (None, Some(AuthenticationError::InvalidToken));
    }

    let user_result = match users::Model::find_by_pid(db, &claims.pid).await {
      Ok(user) => user,
      Err(_) => return (None, Some(AuthenticationError::InvalidClaims)),
    };

    if !user_result.is_access_key_verified {
      return (None, Some(AuthenticationError::AccessKeyNotVerified));
    }

    (Some(user_result), None)
  }

  fn ensure_not_completed(&self) -> Result<(), MyErrors> {
    if self.complete {
      return Err(UnexpectedError::should_not_happen().into());
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;
  use uuid::Uuid;

  use super::*;
  use crate::auth::jwt::{JwtService, TOKEN_TYPE_AUTH, TOKEN_TYPE_PASSWORD_RESET};
  use crate::auth::testing::{setup_tx, user_factory::UserFactory};

  const TEST_SECRET: &str = "test_secret_for_auth_context";

  static DUMMY_USER: LazyLock<users::Model> = LazyLock::new(|| users::Model {
    id: 0,
    pid: Uuid::nil(),
    email: "dummy@test.com".to_string(),
    password: "hash".to_string(),
    phone_number: "0000000000".to_string(),
    first_name: "Dummy".to_string(),
    last_name: "User".to_string(),
    access_key: None,
    is_access_key_verified: false,
    created_at: chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00").unwrap(),
    updated_at: chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00").unwrap(),
  });

  fn a_token(pid: &str, token_type: &str) -> String {
    JwtService::new(TEST_SECRET).generate_token(pid, token_type, 3600).unwrap()
  }

  fn a_fresh_context() -> AuthContext<'static> {
    AuthContext::for_user(&DUMMY_USER)
  }

  fn a_completed_context() -> AuthContext<'static> {
    AuthContext {
      complete: true,
      ..a_fresh_context()
    }
  }

  // =========================================================================

  mod authorized {
    use super::*;

    mod when_not_yet_completed {
      use super::*;

      #[test]
      fn then_ok_is_returned_and_context_is_marked_as_authorized() {
        // Given
        let mut ctx = a_fresh_context();

        // When
        let result = ctx.authorized();

        // Then
        assert!(result.is_ok());
        assert!(ctx.authorized);
      }
    }

    mod when_already_completed {
      use super::*;

      #[test]
      fn then_a_should_not_happen_error_is_returned() {
        // Given
        let mut ctx = a_completed_context();

        // When
        let result = ctx.authorized();

        // Then
        assert_eq!(result.unwrap_err(), UnexpectedError::should_not_happen().into());
      }
    }
  }

  // =========================================================================

  mod not_authorized {
    use super::*;

    mod when_not_yet_completed {
      use super::*;

      #[test]
      fn then_ok_is_returned_and_the_provided_error_is_stored() {
        // Given
        let mut ctx = a_fresh_context();

        // When
        let result = ctx.not_authorized(Some(AuthenticationError::AccessDenied(None).into()));

        // Then
        assert!(result.is_ok());
        assert_eq!(ctx.error.unwrap(), AuthenticationError::AccessDenied(None).into());
      }
    }

    mod when_the_context_already_has_an_error {
      use super::*;

      #[test]
      fn then_the_original_error_is_preserved() {
        // Given
        let mut ctx = AuthContext {
          error: Some(AuthenticationError::InvalidToken.into()),
          ..a_fresh_context()
        };

        // When
        ctx
          .not_authorized(Some(AuthenticationError::InvalidCredentials.into()))
          .unwrap();

        // Then
        assert_eq!(ctx.error.unwrap(), AuthenticationError::InvalidToken.into());
      }
    }

    mod when_already_completed {
      use super::*;

      #[test]
      fn then_a_should_not_happen_error_is_returned() {
        // Given
        let mut ctx = a_completed_context();

        // When
        let result = ctx.not_authorized(None);

        // Then
        assert_eq!(result.unwrap_err(), UnexpectedError::should_not_happen().into());
      }
    }
  }

  // =========================================================================

  mod complete {
    use super::*;

    mod when_authorized {
      use super::*;

      #[test]
      fn then_ok_is_returned() {
        // Given
        let mut ctx = AuthContext {
          authorized: true,
          ..a_fresh_context()
        };

        // When
        let result = ctx.complete();

        // Then
        assert!(result.is_ok());
      }
    }

    mod when_not_authorized_and_an_error_was_set {
      use super::*;

      #[test]
      fn then_that_error_is_returned() {
        // Given
        let mut ctx = AuthContext {
          error: Some(AuthenticationError::InvalidToken.into()),
          ..a_fresh_context()
        };

        // When
        let result = ctx.complete();

        // Then
        assert_eq!(result.unwrap_err(), AuthenticationError::InvalidToken.into());
      }
    }

    mod when_not_authorized_and_no_error_was_set {
      use super::*;

      #[test]
      fn then_a_generic_access_denied_error_is_returned() {
        // Given
        let mut ctx = a_fresh_context();

        // When
        let result = ctx.complete();

        // Then
        assert_eq!(result.unwrap_err(), AuthenticationError::AccessDenied(None).into());
      }
    }

    mod when_already_completed {
      use super::*;

      #[test]
      fn then_a_should_not_happen_error_is_returned() {
        // Given
        let mut ctx = a_completed_context();

        // When
        let result = ctx.complete();

        // Then
        assert_eq!(result.unwrap_err(), UnexpectedError::should_not_happen().into());
      }
    }
  }

  // =========================================================================

  mod validate_auth_header {
    use super::*;

    mod when_the_authorization_header_has_no_bearer_prefix {

      use super::*;

      #[tokio::test]
      async fn then_a_missing_token_error_is_returned() {
        // Given
        let db = setup_tx().await;
        let header = "Basic c29tZS1jcmVkZW50aWFscw==";

        // When
        let (user, error) = AuthContext::validate_auth_header(header, &db, TEST_SECRET).await;

        // Then
        assert!(user.is_none());
        assert!(matches!(error, Some(AuthenticationError::MissingToken)));
      }
    }

    mod when_the_bearer_token_is_malformed {
      use super::*;

      #[tokio::test]
      async fn then_an_invalid_token_error_is_returned() {
        // Given
        let db = setup_tx().await;
        let header = "Bearer not.a.real.jwt";

        // When
        let (user, error) = AuthContext::validate_auth_header(header, &db, TEST_SECRET).await;

        // Then
        assert!(user.is_none());
        assert!(matches!(error, Some(AuthenticationError::InvalidToken)));
      }
    }

    mod when_the_token_type_is_not_auth {
      use super::*;

      #[tokio::test]
      async fn then_an_invalid_token_error_is_returned() {
        // Given
        let db = setup_tx().await;
        let pid = Uuid::new_v4().to_string();
        let token = a_token(&pid, TOKEN_TYPE_PASSWORD_RESET);
        let header = format!("Bearer {token}");

        // When
        let (user, error) = AuthContext::validate_auth_header(&header, &db, TEST_SECRET).await;

        // Then
        assert!(user.is_none());
        assert!(matches!(error, Some(AuthenticationError::InvalidToken)));
      }
    }

    mod when_the_user_does_not_exist_in_the_database {
      use super::*;

      #[tokio::test]
      async fn then_an_invalid_claims_error_is_returned() {
        // Given — empty transaction, no users exist
        let db = setup_tx().await;
        let pid = Uuid::new_v4().to_string();
        let token = a_token(&pid, TOKEN_TYPE_AUTH);
        let header = format!("Bearer {token}");

        // When
        let (user, error) = AuthContext::validate_auth_header(&header, &db, TEST_SECRET).await;

        // Then
        assert!(user.is_none());
        assert!(matches!(error, Some(AuthenticationError::InvalidClaims)));
      }
    }

    mod when_the_user_has_not_verified_their_access_key {
      use super::*;

      #[tokio::test]
      async fn then_an_access_key_not_verified_error_is_returned() {
        // Given
        let db = setup_tx().await;
        let user = UserFactory::new().unverified().create(&db).await;
        let token = a_token(&user.pid.to_string(), TOKEN_TYPE_AUTH);
        let header = format!("Bearer {token}");

        // When
        let (returned_user, error) = AuthContext::validate_auth_header(&header, &db, TEST_SECRET).await;

        // Then
        assert!(returned_user.is_none());
        assert!(matches!(error, Some(AuthenticationError::AccessKeyNotVerified)));
      }
    }

    mod when_the_token_is_valid_and_the_user_is_verified {
      use super::*;

      #[tokio::test]
      async fn then_the_user_is_returned_with_no_error() {
        // Given
        let db = setup_tx().await;
        let user = UserFactory::new().create(&db).await;
        let token = a_token(&user.pid.to_string(), TOKEN_TYPE_AUTH);
        let header = format!("Bearer {token}");

        // When
        let (returned_user, error) = AuthContext::validate_auth_header(&header, &db, TEST_SECRET).await;

        // Then
        assert!(error.is_none());
        let returned_user = returned_user.expect("should return the authenticated user");
        assert_eq!(returned_user.pid, user.pid);
      }
    }
  }
}
