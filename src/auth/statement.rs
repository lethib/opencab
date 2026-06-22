use sea_orm::DatabaseConnection;

use crate::{
  auth::{context::AuthContext, resource::Resource},
  models::my_errors::{
    authentication_error::AuthenticationError, unexpected_error::UnexpectedError, MyErrors,
  },
};

pub struct AuthStatement<'user, 'db> {
  auth_context: AuthContext<'user>,
  db: &'db DatabaseConnection,
  is_empty: bool,
  ok_so_far: bool,
  error: Option<MyErrors>,
}

impl<'user, 'db> AuthStatement<'user, 'db> {
  pub(super) fn new(auth_context: AuthContext<'user>, db: &'db DatabaseConnection) -> Self {
    Self {
      auth_context,
      db,
      is_empty: true,
      ok_so_far: true,
      error: None,
    }
  }

  pub fn run_complete(mut self) -> Result<(), MyErrors> {
    if self.is_empty {
      return Err(UnexpectedError::ShouldNotHappen.into());
    }

    if self.ok_so_far {
      self.auth_context.authorized()?;
    } else {
      self.auth_context.not_authorized(self.error.take())?;
    }

    self.auth_context.complete()
  }

  pub fn non_authenticated_user(self) -> Self {
    self.check(|_| true, None)
  }

  pub fn authenticated_user(self) -> Self {
    self.check(|_| true, Some(AuthenticationError::InvalidToken.into()))
  }

  pub async fn user_owning_resource<T: Resource>(self, resource: &T) -> Self {
    let is_owned = resource
      .is_owned_by_user(self.auth_context.current_user.id, self.db)
      .await;

    self.check(
      |_| is_owned,
      Some(AuthenticationError::AccessDenied(Some(resource.resource_name())).into()),
    )
  }

  #[allow(dead_code)]
  pub fn or<F>(mut self, check_fn: F) -> Self
  where
    F: FnOnce(Self) -> Self,
  {
    if self.ok_so_far {
      return self;
    }

    // Reset state for OR operation: if previous checks failed,
    // give this branch a clean slate to succeed
    self.ok_so_far = true;
    self.error = None;

    check_fn(self)
  }

  pub fn check<F>(mut self, predicate: F, error: Option<MyErrors>) -> Self
  where
    F: FnOnce(&Self) -> bool,
  {
    self.is_empty = false;

    if !self.ok_so_far {
      return self;
    }

    if predicate(&self) {
      self
    } else {
      self.ok_so_far = false;
      self.error = error;
      self
    }
  }
}
