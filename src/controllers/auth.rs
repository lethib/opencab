use crate::{
  app_state::{AppState, WorkerJob},
  auth::{
    jwt::{JwtService, TOKEN_TYPE_AUTH, TOKEN_TYPE_PASSWORD_RESET},
    statement::AuthStatement,
  },
  config::Config,
  db::DB,
  middleware::auth::AuthenticatedUser,
  models::{
    _entities::users,
    my_errors::{
      application_error::ApplicationError, authentication_error::AuthenticationError,
      unexpected_error::UnexpectedError, MyErrors,
    },
    users::{LoginParams, RegisterParams},
  },
  services::{self},
  views::auth::{CurrentResponse, LoginResponse},
  workers::mailer::args::EmailArgs,
};
use axum::{
  debug_handler,
  extract::State,
  http::{self, StatusCode},
  Json,
};
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgotParams {
  pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetParams {
  pub token: String,
  pub password: String,
}

#[debug_handler]
pub async fn register(
  State(_state): State<AppState>,
  authorize: AuthStatement,
  Json(params): Json<RegisterParams>,
) -> Result<Json<()>, MyErrors> {
  authorize.non_authenticated_user().run_complete()?;

  users::Model::create_with_password(DB::get(), &params).await?;

  Ok(Json(()))
}

#[debug_handler]
pub async fn forgot(
  State(state): State<AppState>,
  authorize: AuthStatement,
  Json(params): Json<ForgotParams>,
) -> Result<http::StatusCode, MyErrors> {
  authorize.non_authenticated_user().run_complete()?;

  let Ok(user) = users::Model::find_by_email(DB::get(), &params.email).await else {
    return Ok(http::StatusCode::NO_CONTENT);
  };

  let jwt_service = JwtService::new(&Config::get().jwt.secret);
  let secured_token = jwt_service
    .generate_token(&user.pid.to_string(), TOKEN_TYPE_PASSWORD_RESET, 900)
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  let secured_url = format!(
    "{}/reset_password?access_token={}",
    Config::get().app.base_url,
    secured_token
  );

  let email_args = EmailArgs::new_text(
    user.email,
    "Réinitialisation du mot de passe".to_string(),
    format!(
      "Bonjour,\n\nVoici le lien pour réinitialiser votre mot de passe: {}",
      secured_url
    ),
  );

  state
    .worker_transmitter
    .send(WorkerJob::Email(email_args))
    .await?;

  Ok(http::StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn reset(
  State(_state): State<AppState>,
  authorize: AuthStatement,
  Json(params): Json<ResetParams>,
) -> Result<Json<()>, MyErrors> {
  authorize.non_authenticated_user().run_complete()?;

  let jwt_service = JwtService::new(&Config::get().jwt.secret);
  let claims = jwt_service
    .validate_token(&params.token)
    .map_err(|_| AuthenticationError::InvalidToken)?;

  if claims.token_type != TOKEN_TYPE_PASSWORD_RESET {
    return Err(AuthenticationError::InvalidToken.into());
  }

  let user = users::Model::find_by_pid(DB::get(), &claims.pid)
    .await
    .map_err(|_| AuthenticationError::InvalidClaims)?;

  user
    .0
    .into_active_model()
    .update_password(DB::get(), &params.password)
    .await?;

  Ok(Json(()))
}

/// Creates a user login and returns a token
#[debug_handler]
pub async fn login(
  State(_state): State<AppState>,
  authorize: AuthStatement,
  Json(params): Json<LoginParams>,
) -> Result<Json<LoginResponse>, MyErrors> {
  authorize.non_authenticated_user().run_complete()?;

  let user = users::Model::find_by_email(DB::get(), &params.email)
    .await
    .map_err(|_| AuthenticationError::InvalidCredentials)?;

  let valid = user.verify_password(&params.password);

  if !valid {
    return Err(AuthenticationError::InvalidCredentials.into());
  }

  if !user.is_access_key_verified {
    return Err(MyErrors {
      code: StatusCode::SEE_OTHER,
      msg: "access_key_needs_to_be_verified".to_string(),
    });
  }

  let jwt_service = JwtService::new(&Config::get().jwt.secret);
  let token = jwt_service
    .generate_token(
      &user.pid.to_string(),
      TOKEN_TYPE_AUTH,
      Config::get().jwt.expiration,
    )
    .map_err(|_| MyErrors {
      code: StatusCode::UNAUTHORIZED,
      msg: "Failed to generate token".to_string(),
    })?;

  Ok(Json(LoginResponse::new(&user, &token)))
}

/// Get current authenticated user
#[debug_handler]
pub async fn me(
  State(_state): State<AppState>,
  AuthenticatedUser(current_user, business_info): AuthenticatedUser,
) -> Result<Json<CurrentResponse>, MyErrors> {
  Ok(Json(CurrentResponse::new(&(current_user, business_info))))
}

#[derive(Deserialize)]
pub struct CheckAccessKeyParams {
  access_key: String,
  user_email: String,
}

#[debug_handler]
pub async fn check_access_key(
  State(_state): State<AppState>,
  authorize: AuthStatement,
  Json(params): Json<CheckAccessKeyParams>,
) -> Result<Json<serde_json::Value>, MyErrors> {
  authorize.non_authenticated_user().run_complete()?;

  let user = users::Model::find_by_email(DB::get(), &params.user_email)
    .await
    .map_err(|_| UnexpectedError::ShouldNotHappen)?;

  if services::user::check_access_key(&user, params.access_key) {
    users::ActiveModel::enable_access(&mut user.clone().into_active_model(), DB::get()).await?;

    let jwt_service = JwtService::new(&Config::get().jwt.secret);
    let token = jwt_service
      .generate_token(
        &user.pid.to_string(),
        TOKEN_TYPE_AUTH,
        Config::get().jwt.expiration,
      )
      .map_err(|error| UnexpectedError::new(error.to_string()))?;

    return Ok(Json(serde_json::json!({ "token": token })));
  }

  Err(ApplicationError::new("access_key_not_recognized").into())
}
