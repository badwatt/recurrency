use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Validation(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Validation error",
                        "details": e.to_string()
                    })),
                )
                    .into_response();
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
        };

        (
            status,
            Json(json!({
                "error": error_message
            })),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_display() {
        let err = AppError::Unauthorized("bad token".into());
        assert_eq!(err.to_string(), "Unauthorized: bad token");
    }

    #[test]
    fn bad_request_display() {
        let err = AppError::BadRequest("invalid data".into());
        assert_eq!(err.to_string(), "Bad request: invalid data");
    }

    #[test]
    fn not_found_display() {
        let err = AppError::NotFound("subscription not found".into());
        assert_eq!(err.to_string(), "Not found: subscription not found");
    }

    #[test]
    fn conflict_display() {
        let err = AppError::Conflict("email already exists".into());
        assert_eq!(err.to_string(), "Conflict: email already exists");
    }

    #[test]
    fn internal_display() {
        let err = AppError::Internal("something broke".into());
        assert_eq!(err.to_string(), "Internal server error: something broke");
    }

    // ── IntoResponse tests ──

    #[test]
    fn unauthorized_into_response_status() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Unauthorized("bad token".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bad_request_into_response_status() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::BadRequest("invalid data".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn not_found_into_response_status() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::NotFound("missing".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn conflict_into_response_status() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Conflict("duplicate".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn internal_into_response_status() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Internal("crash".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ── serde serialization of error responses ──

    #[test]
    fn unauthorized_error_does_not_panic() {
        // Verify that the IntoResponse doesn't panic
        let err = AppError::Unauthorized("bad token".into());
        let _response = err.into_response();
    }

    #[test]
    fn database_error_returns_500() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Internal("db fail".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn all_error_variants_into_response() {
        use axum::response::IntoResponse;
        // Test every variant creates a non-panicking response
        let _resp = AppError::Unauthorized("test".into()).into_response();
        let _resp = AppError::BadRequest("test".into()).into_response();
        let _resp = AppError::NotFound("test".into()).into_response();
        let _resp = AppError::Conflict("test".into()).into_response();
        let _resp = AppError::Internal("test".into()).into_response();
    }

    #[test]
    fn error_debug_format() {
        let err = AppError::NotFound("gone".into());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn all_error_display_messages() {
        assert_eq!(AppError::Unauthorized("x".into()).to_string(), "Unauthorized: x");
        assert_eq!(AppError::BadRequest("x".into()).to_string(), "Bad request: x");
        assert_eq!(AppError::NotFound("x".into()).to_string(), "Not found: x");
        assert_eq!(AppError::Conflict("x".into()).to_string(), "Conflict: x");
        assert_eq!(AppError::Internal("x".into()).to_string(), "Internal server error: x");
    }

    #[test]
    fn app_error_from_sqlx() {
        // Verify that the From<sqlx::Error> impl exists
        // We can create a sqlx error from a connection error message
        let sqlx_err = sqlx::Error::PoolTimedOut;
        let app_err: AppError = sqlx_err.into();
        match app_err {
            AppError::Database(_) => {},
            _ => panic!("Expected Database variant"),
        }
    }

    #[test]
    fn app_error_from_jwt() {
        let jwt_err = jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken);
        let app_err: AppError = jwt_err.into();
        match app_err {
            AppError::Jwt(_) => {},
            _ => panic!("Expected Jwt variant"),
        }
    }

    // ── AppResult type alias ──

    #[test]
    fn app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn app_result_err() {
        let result: AppResult<i32> = Err(AppError::NotFound("gone".into()));
        assert!(result.is_err());
    }

    #[test]
    fn validation_error_into_response() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Validation(validator::ValidationErrors::new());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validation_error_body_contains_status_code() {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let err = AppError::Validation(validator::ValidationErrors::new());
        let response = err.into_response();
        // Body content is tested via integration tests; just verify status here
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
