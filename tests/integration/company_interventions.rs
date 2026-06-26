use axum::{
  body::Body,
  http::{Request, StatusCode},
};
use opencab::{
  auth::jwt::{JwtService, TOKEN_TYPE_AUTH},
  config::Config,
  models::_entities::{company_interventions, users},
};
use sea_orm::EntityTrait;
use tower::ServiceExt;

use crate::common::setup_http;
use crate::factories::{company::CompanyFactory, company_intervention::CompanyInterventionFactory, user::UserFactory};

fn token_for(user: &users::Model, config: &Config) -> String {
  JwtService::new(&config.jwt.secret)
    .generate_token(&user.pid.to_string(), TOKEN_TYPE_AUTH, 3600)
    .unwrap()
}

// ============================================================

mod delete_an_intervention {
  use super::*;

  mod when_the_user_is_not_authenticated {
    use super::*;

    #[tokio::test]
    async fn then_it_returns_401() {
      // Given
      let app = setup_http().await;
      let user = UserFactory::new().create(&app.db).await;
      let company = CompanyFactory::new().create_for_user(&app.db, user.id).await;
      let intervention = CompanyInterventionFactory::new().create(&app.db, user.id, company.id).await;

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/companies/{}/interventions/{}", company.id, intervention.id))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
  }

  // ============================================================

  mod when_the_company_does_not_exist {
    use super::*;

    #[tokio::test]
    async fn then_it_returns_404() {
      // Given
      let app = setup_http().await;
      let user = UserFactory::new().create(&app.db).await;
      let token = token_for(&user, &app.config);

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri("/api/companies/99999/interventions/99999")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
  }

  // ============================================================

  mod when_the_intervention_does_not_exist {
    use super::*;

    #[tokio::test]
    async fn then_it_returns_404() {
      // Given
      let app = setup_http().await;
      let user = UserFactory::new().create(&app.db).await;
      let company = CompanyFactory::new().create_for_user(&app.db, user.id).await;
      let token = token_for(&user, &app.config);

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/companies/{}/interventions/99999", company.id))
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
  }

  // ============================================================

  mod when_the_intervention_belongs_to_another_company {
    use super::*;

    #[tokio::test]
    async fn then_it_returns_404() {
      // Given
      let app = setup_http().await;
      let user = UserFactory::new().create(&app.db).await;
      let company = CompanyFactory::new().create_for_user(&app.db, user.id).await;
      let other_company = CompanyFactory::new().create_for_user(&app.db, user.id).await;
      let intervention = CompanyInterventionFactory::new().create(&app.db, user.id, other_company.id).await;
      let token = token_for(&user, &app.config);

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/companies/{}/interventions/{}", company.id, intervention.id))
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::NOT_FOUND);
      let still_there = company_interventions::Entity::find_by_id(intervention.id)
        .one(&app.db)
        .await
        .unwrap();
      assert!(still_there.is_some(), "intervention should not have been deleted");
    }
  }

  // ============================================================

  mod when_the_company_belongs_to_another_user {
    use super::*;

    #[tokio::test]
    async fn then_it_returns_403() {
      // Given
      let app = setup_http().await;
      let owner = UserFactory::new().create(&app.db).await;
      let other = UserFactory::new().create(&app.db).await;
      let company = CompanyFactory::new().create_for_user(&app.db, owner.id).await;
      let intervention = CompanyInterventionFactory::new().create(&app.db, owner.id, company.id).await;
      let token = token_for(&other, &app.config);

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/companies/{}/interventions/{}", company.id, intervention.id))
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::FORBIDDEN);
      let still_there = company_interventions::Entity::find_by_id(intervention.id)
        .one(&app.db)
        .await
        .unwrap();
      assert!(still_there.is_some(), "intervention should not have been deleted");
    }
  }

  // ============================================================

  mod when_the_user_owns_the_intervention {
    use super::*;

    #[tokio::test]
    async fn then_it_deletes_the_intervention_and_returns_204() {
      // Given
      let app = setup_http().await;
      let user = UserFactory::new().create(&app.db).await;
      let company = CompanyFactory::new().create_for_user(&app.db, user.id).await;
      let intervention = CompanyInterventionFactory::new().create(&app.db, user.id, company.id).await;
      let token = token_for(&user, &app.config);

      // When
      let response = app
        .router
        .oneshot(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/companies/{}/interventions/{}", company.id, intervention.id))
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      // Then
      assert_eq!(response.status(), StatusCode::NO_CONTENT);
      let deleted = company_interventions::Entity::find_by_id(intervention.id)
        .one(&app.db)
        .await
        .unwrap();
      assert!(deleted.is_none(), "intervention should have been deleted");
    }
  }
}
