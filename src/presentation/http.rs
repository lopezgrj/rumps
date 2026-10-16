use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{SecondsFormat, Utc};
use serde_json::{Value, json};
use sqlx::AnyPool;
use std::sync::Arc;
use tracing::warn;

use crate::infrastructure::oidc::OidcAuthenticator;

pub fn router(pool: AnyPool, authenticator: Option<Arc<OidcAuthenticator>>) -> Router {
    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/fhir/metadata", get(capability_statement))
        .route("/openapi.json", get(openapi_document))
        .with_state(pool)
        .layer(from_fn_with_state(authenticator, authenticate))
}

async fn authenticate(
    State(authenticator): State<Option<Arc<OidcAuthenticator>>>,
    mut request: Request,
    next: Next,
) -> Response {
    if is_public_route(request.uri().path()) {
        return next.run(request).await;
    }

    let Some(authenticator) = authenticator else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "authentication_not_configured" })),
        )
            .into_response();
    };

    let Some(token) = bearer_token(&request) else {
        return unauthorized();
    };

    match authenticator.verify(token) {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(_) => unauthorized(),
    }
}

fn is_public_route(path: &str) -> bool {
    matches!(
        path,
        "/health/live" | "/health/ready" | "/fhir/metadata" | "/openapi.json"
    )
}

fn bearer_token(request: &Request) -> Option<&str> {
    let value = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() && !token.contains(' '))
        .then_some(token)
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"rumps\", error=\"invalid_token\""),
        )],
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}

async fn liveness() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn readiness(State(pool): State<AnyPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ready" }))),
        Err(error) => {
            warn!(%error, "database readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "not_ready" })),
            )
        }
    }
}

async fn capability_statement() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/fhir+json"),
    );

    (
        headers,
        Json(json!({
            "resourceType": "CapabilityStatement",
            "status": "active",
            "date": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            "kind": "instance",
            "name": "RumpsApi",
            "title": "RUMPS API",
            "fhirVersion": "4.0.1",
            "format": ["json"],
            "implementation": {
                "description": "RUMPS API foundation. VistA resources and interactions are not yet implemented."
            },
            "rest": [{
                "mode": "server",
                "resource": []
            }]
        })),
    )
}

async fn openapi_document() -> Json<Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "RUMPS API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Current API foundation. No VistA clinical resources are exposed yet. Protected operations require OIDC bearer tokens from an allowlisted service client."
        },
        "components": {
            "securitySchemes": {
                "oidcBearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "servers": [{ "url": "/" }],
        "paths": {
            "/health/live": {
                "get": {
                    "operationId": "getLiveness",
                    "summary": "Check process liveness",
                    "responses": {
                        "200": { "description": "The process is running." }
                    }
                }
            },
            "/health/ready": {
                "get": {
                    "operationId": "getReadiness",
                    "summary": "Check database readiness",
                    "responses": {
                        "200": { "description": "The database is reachable." },
                        "503": { "description": "The database is unavailable." }
                    }
                }
            },
            "/fhir/metadata": {
                "get": {
                    "operationId": "getFhirCapabilityStatement",
                    "summary": "Read FHIR server capabilities",
                    "responses": {
                        "200": {
                            "description": "FHIR R4 CapabilityStatement.",
                            "content": {
                                "application/fhir+json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "resourceType": { "const": "CapabilityStatement" },
                                            "fhirVersion": { "const": "4.0.1" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request};
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn liveness_reports_service_state() -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let body: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["status"], "ok");
        Ok(())
    }

    #[tokio::test]
    async fn capability_statement_advertises_only_implemented_interactions()
    -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/fhir/metadata")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE),
            Some(&HeaderValue::from_static("application/fhir+json"))
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let body: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["resourceType"], "CapabilityStatement");
        assert_eq!(body["fhirVersion"], "4.0.1");
        assert_eq!(
            body["rest"][0]["resource"].as_array().map(Vec::len),
            Some(0)
        );
        Ok(())
    }

    #[tokio::test]
    async fn readiness_checks_database_connectivity() -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect("sqlite::memory:").await?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let body: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["status"], "ready");
        Ok(())
    }

    #[tokio::test]
    async fn openapi_document_lists_only_live_routes() -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let body: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["openapi"], "3.1.0");
        assert!(body["paths"]["/fhir/metadata"]["get"].is_object());
        assert!(body["paths"]["/Patient"].is_null());
        Ok(())
    }

    #[tokio::test]
    async fn protected_routes_fail_closed_without_oidc_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        Ok(())
    }

    #[tokio::test]
    async fn public_health_routes_do_not_require_oidc() -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let response = router(pool, None)
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn protected_routes_reject_missing_bearer_tokens()
    -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect_lazy("sqlite::memory:")?;
        let authenticator = Arc::new(OidcAuthenticator::empty_test_instance());
        let response = router(pool, Some(authenticator))
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(axum::body::Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE),
            Some(&HeaderValue::from_static(
                "Bearer realm=\"rumps\", error=\"invalid_token\""
            ))
        );
        Ok(())
    }
}
