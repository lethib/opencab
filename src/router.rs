use crate::{app_state::AppState, controllers, middleware::auth::authenticated_request};
use axum::{
  http::{HeaderName, Method},
  middleware,
  routing::{delete, get, post, put},
  Router,
};
use tower_http::{
  cors::CorsLayer,
  services::{ServeDir, ServeFile},
  trace::TraceLayer,
};

pub fn create_router(state: AppState) -> Router {
  // Public routes (no authentication required)
  let public_routes = Router::new()
    .route("/api/auth/register", post(controllers::auth::register))
    .route("/api/auth/login", post(controllers::auth::login))
    .route("/api/auth/forgot", post(controllers::auth::forgot))
    .route("/api/auth/reset", post(controllers::auth::reset))
    .route(
      "/api/auth/_check_access_key",
      post(controllers::auth::check_access_key),
    );

  // Protected routes (require authentication)
  let protected_routes = Router::new()
    // Auth routes
    .route("/api/auth/me", get(controllers::auth::me))
    // Patient routes
    .route("/api/patient/create", post(controllers::patient::create))
    .route("/api/patient/{patient_id}", get(controllers::patient::get))
    .route(
      "/api/patient/{patient_id}",
      put(controllers::patient::update),
    )
    .route(
      "/api/patient/{patient_id}",
      delete(controllers::patient::delete),
    )
    .route("/api/patient/_search", get(controllers::patient::search))
    .route(
      "/api/patient/{patient_id}/_generate_invoice",
      post(controllers::patient::generate_invoice),
    )
    .route(
      "/api/patient/{patient_id}/medical_appointments",
      get(controllers::patient::get_medical_appointments)
        .post(controllers::medical_appointment::create),
    )
    .route(
      "/api/patient/{patient_id}/medical_appointments/{appointment_id}",
      put(controllers::medical_appointment::update)
        .delete(controllers::medical_appointment::delete),
    )
    // User routes
    .route(
      "/api/user/_save_business_information",
      post(controllers::user::save_business_info),
    )
    .route(
      "/api/user/_extract_medical_appointments",
      post(controllers::user::extract_medical_appointments),
    )
    .route(
      "/api/user/_generate_accountability",
      post(controllers::user::generate_accountability),
    )
    .route("/api/user/my_offices", get(controllers::user::my_offices))
    .route(
      "/api/user/signature/_get_url",
      post(controllers::user::get_signature_url),
    )
    .route(
      "/api/user/signature/_upload",
      post(controllers::user::upload_signature),
    )
    // Practitioner office routes
    .route(
      "/api/practitioner_office/create",
      post(controllers::practitioner_office::create),
    )
    .route(
      "/api/practitioner_office/{office_id}",
      put(controllers::practitioner_office::update),
    )
    .route(
      "/api/practitioner_office/{office_id}",
      delete(controllers::practitioner_office::destroy),
    )
    // Apply auth middleware to all protected routes
    .layer(middleware::from_fn_with_state(
      state.clone(),
      authenticated_request,
    ));

  // Build CORS layer from configuration
  let cors_config = &state.config.cors;
  let mut cors_layer = CorsLayer::new();

  if cors_config.allow_origins.contains(&"*".to_string()) {
    cors_layer = cors_layer.allow_origin(tower_http::cors::Any);
  } else {
    let origins: Vec<_> = cors_config
      .allow_origins
      .iter()
      .filter_map(|origin| origin.parse().ok())
      .collect();
    cors_layer = cors_layer.allow_origin(origins);
  }

  let methods: Vec<Method> = cors_config
    .allow_methods
    .iter()
    .filter_map(|method| method.parse().ok())
    .collect();
  cors_layer = cors_layer.allow_methods(methods);

  let headers: Vec<HeaderName> = cors_config
    .allow_headers
    .iter()
    .filter_map(|header| header.parse().ok())
    .collect();
  cors_layer = cors_layer.allow_headers(headers);

  // Combine all routes
  Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    // Serve static files for frontend
    .fallback_service(
      ServeDir::new("frontend/dist").fallback(ServeFile::new("frontend/dist/index.html")),
    )
    // HTTP request tracing middleware
    .layer(TraceLayer::new_for_http())
    .layer(cors_layer)
    .with_state(state)
}
