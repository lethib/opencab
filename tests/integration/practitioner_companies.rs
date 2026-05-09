use axum::{
  body::Body,
  http::{Request, StatusCode},
};
use opencab::{
  auth::jwt::{JwtService, TOKEN_TYPE_AUTH},
  config::Config,
  db::DB,
  models::_entities::{practitioner_companies, users},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{setup_http, RUNTIME};
use crate::factories::{company::CompanyFactory, user::UserFactory};

fn token_for(user: &users::Model) -> String {
  JwtService::new(&Config::get().jwt.secret)
    .generate_token(&user.pid.to_string(), TOKEN_TYPE_AUTH, 3600)
    .unwrap()
}

// ============================================================

mod create_a_company {
  use super::*;

  mod when_the_user_is_not_authenticated {
    use super::*;

    #[test]
    #[serial]
    fn then_it_returns_401() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("POST")
              .uri("/api/companies")
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "My Company",
                  "contact_email": "contact@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
      });
    }
  }

  // ============================================================

  mod when_the_user_is_authenticated_with_valid_params {
    use super::*;

    #[test]
    #[serial]
    fn then_it_creates_the_company_and_returns_204() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let token = token_for(&user);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("POST")
              .uri("/api/companies")
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "My Company",
                  "contact_email": "contact@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let company = practitioner_companies::Entity::find()
          .filter(practitioner_companies::Column::UserId.eq(user.id))
          .one(DB::get())
          .await
          .unwrap()
          .expect("company should have been created");
        assert_eq!(company.name, "My Company");
        assert_eq!(company.contact_email, "contact@example.com");
        assert!(company.address_country.is_none());
      });
    }
  }

  // ============================================================

  mod when_the_user_is_authenticated_with_a_valid_address {
    use super::*;

    #[test]
    #[serial]
    fn then_address_country_is_set_to_france() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let token = token_for(&user);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("POST")
              .uri("/api/companies")
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "My Company",
                  "contact_email": "contact@example.com",
                  "address_line_1": "1 rue de la Paix",
                  "address_zip_code": "75001"
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let company = practitioner_companies::Entity::find()
          .filter(practitioner_companies::Column::UserId.eq(user.id))
          .one(DB::get())
          .await
          .unwrap()
          .expect("company should have been created");
        assert_eq!(company.address_country.as_deref(), Some("FRANCE"));
      });
    }
  }

  // ============================================================

  mod when_the_user_is_authenticated_with_an_invalid_zip_code {
    use super::*;

    #[test]
    #[serial]
    fn then_it_returns_422() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let token = token_for(&user);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("POST")
              .uri("/api/companies")
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "My Company",
                  "contact_email": "contact@example.com",
                  "address_line_1": "1 rue de la Paix",
                  "address_zip_code": "INVALID"
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
      });
    }
  }
}

// ============================================================

mod update_a_company {
  use super::*;

  mod when_the_user_is_not_authenticated {
    use super::*;

    #[test]
    #[serial]
    fn then_it_returns_401() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let company = CompanyFactory::new().create_for_user(DB::get(), user.id).await;

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("PUT")
              .uri(format!("/api/companies/{}", company.id))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "Updated Name",
                  "contact_email": "updated@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
      });
    }
  }

  // ============================================================

  mod when_the_company_does_not_exist {
    use super::*;

    #[test]
    #[serial]
    fn then_it_returns_404() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let token = token_for(&user);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("PUT")
              .uri("/api/companies/99999")
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "Updated Name",
                  "contact_email": "updated@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
      });
    }
  }

  // ============================================================

  mod when_the_company_belongs_to_another_user {
    use super::*;

    #[test]
    #[serial]
    fn then_it_returns_403() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let owner = UserFactory::new().create(DB::get()).await;
        let other = UserFactory::new().email("other@test.com").create(DB::get()).await;
        let company = CompanyFactory::new().create_for_user(DB::get(), owner.id).await;
        let token = token_for(&other);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("PUT")
              .uri(format!("/api/companies/{}", company.id))
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "Updated Name",
                  "contact_email": "updated@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
      });
    }
  }

  // ============================================================

  mod when_the_user_owns_the_company {
    use super::*;

    #[test]
    #[serial]
    fn then_it_updates_the_company_and_returns_204() {
      RUNTIME.block_on(async {
        // Given
        let router = setup_http().await;
        let user = UserFactory::new().create(DB::get()).await;
        let company = CompanyFactory::new().create_for_user(DB::get(), user.id).await;
        let token = token_for(&user);

        // When
        let response = router
          .oneshot(
            Request::builder()
              .method("PUT")
              .uri(format!("/api/companies/{}", company.id))
              .header("Authorization", format!("Bearer {token}"))
              .header("Content-Type", "application/json")
              .body(Body::from(
                json!({
                  "name": "Updated Name",
                  "contact_email": "updated@example.com",
                  "address_line_1": null,
                  "address_zip_code": null
                })
                .to_string(),
              ))
              .unwrap(),
          )
          .await
          .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let updated = practitioner_companies::Entity::find_by_id(company.id)
          .one(DB::get())
          .await
          .unwrap()
          .expect("company should still exist");
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.contact_email, "updated@example.com");
      });
    }
  }
}
