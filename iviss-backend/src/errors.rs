use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    DatabaseError,
    NotFound,
    BadRequest,
    TooManyRequests,
    ExternalApiFailure,
    InternalError,
}

#[allow(dead_code)]
#[derive(Serialize, ToSchema)]
pub struct AppErrorResponse {
    code: ErrorCode,
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("External API failure: {0}")]
    ExternalApiFailure(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[allow(dead_code)]
impl AppError {
    pub fn database(err: impl Into<sqlx::Error>) -> Self {
        Self::Database(err.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn too_many_requests(msg: impl Into<String>) -> Self {
        Self::TooManyRequests(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn external_api_failure(msg: impl Into<String>) -> Self {
        Self::ExternalApiFailure(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::Internal(anyhow::anyhow!(msg.into()))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(err) => {
                // Log the detailed error for the server operator
                tracing::error!("Database error: {:?}", err);
                // Return a generic error to the client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "Internal Server Error".to_string(),
                )
            }
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                msg.clone(),
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, ErrorCode::Forbidden, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorCode::NotFound, msg.clone()),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorCode::BadRequest, msg.clone())
            }
            AppError::TooManyRequests(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::TooManyRequests,
                msg.clone(),
            ),
            AppError::ExternalApiFailure(msg) => {
                tracing::error!("External API failure: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    ErrorCode::ExternalApiFailure,
                    "External Service Unavailable".to_string(),
                )
            }
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalError,
                    "Internal Server Error".to_string(),
                )
            }
        };

        let body = Json(AppErrorResponse { code, message });

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::Value;

    // Helper to get response body as JSON
    async fn get_body_json(response: Response) -> Value {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body_bytes).unwrap()
    }

    #[tokio::test]
    async fn test_not_found_response() {
        let err = AppError::NotFound("Resource missing".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = get_body_json(response).await;
        assert_eq!(body["code"], "NOT_FOUND");
        assert_eq!(body["message"], "Resource missing");
    }

    #[tokio::test]
    async fn test_unauthorized_response() {
        let err = AppError::Unauthorized("Invalid token".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = get_body_json(response).await;
        assert_eq!(body["code"], "UNAUTHORIZED");
        assert_eq!(body["message"], "Invalid token");
    }

    #[tokio::test]
    async fn test_forbidden_response() {
        let err = AppError::Forbidden("Admin access required".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = get_body_json(response).await;
        assert_eq!(body["code"], "FORBIDDEN");
        assert_eq!(body["message"], "Admin access required");
    }

    #[tokio::test]
    async fn test_external_api_failure_response() {
        let err = AppError::ExternalApiFailure("Timeout connecting to provider".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

        let body = get_body_json(response).await;
        assert_eq!(body["code"], "EXTERNAL_API_FAILURE");
        assert_eq!(body["message"], "External Service Unavailable");
    }

    #[tokio::test]
    async fn test_bad_request_response() {
        let err = AppError::bad_request("missing field");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = get_body_json(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert_eq!(body["message"], "missing field");
    }

    #[tokio::test]
    async fn test_internal_error_response() {
        let err = AppError::internal_error("something broke");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = get_body_json(response).await;
        assert_eq!(body["code"], "INTERNAL_ERROR");
        assert_eq!(body["message"], "Internal Server Error");
    }

    #[tokio::test]
    async fn test_database_error_response() {
        let err = AppError::database(sqlx::Error::RowNotFound);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = get_body_json(response).await;
        assert_eq!(body["code"], "DATABASE_ERROR");
        assert_eq!(body["message"], "Internal Server Error");
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            AppError::unauthorized("fail").to_string(),
            "Authentication failed: fail"
        );
        assert_eq!(AppError::not_found("lost").to_string(), "Not found: lost");
        assert_eq!(AppError::bad_request("bad").to_string(), "Bad request: bad");
    }
}
