//! HTTP Basic Auth extractor for iviss-test-api.
//!
//! Validates credentials against the values loaded from environment variables
//! (`TEST_API_USERNAME` / `TEST_API_PASSWORD`) to mirror the basic auth the
//! real external API enforces.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::STANDARD, Engine};

/// Expected credentials loaded once at startup.
#[derive(Debug, Clone)]
pub struct ApiCredentials {
    pub username: String,
    pub password: String,
}

/// Axum extractor: validates the `Authorization: Basic …` header.
///
/// Returns `401 Unauthorized` (with a `WWW-Authenticate` header) when
/// credentials are absent or incorrect.
pub struct ValidatedBasicAuth;

#[axum::async_trait]
impl<S> FromRequestParts<S> for ValidatedBasicAuth
where
    S: Send + Sync + AsRef<ApiCredentials>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let expected: &ApiCredentials = state.as_ref();

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::Missing)?;

        let encoded = header
            .strip_prefix("Basic ")
            .ok_or(AuthError::Invalid)?;

        let decoded = STANDARD
            .decode(encoded)
            .map_err(|_| AuthError::Invalid)?;

        let credentials = std::str::from_utf8(&decoded).map_err(|_| AuthError::Invalid)?;

        let (username, password) = credentials
            .split_once(':')
            .ok_or(AuthError::Invalid)?;

        if username == expected.username && password == expected.password {
            Ok(ValidatedBasicAuth)
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

/// Auth failure variants.
#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
    Forbidden,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let mut res = StatusCode::UNAUTHORIZED.into_response();
        res.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_static(r#"Basic realm="iviss-test-api""#),
        );
        res
    }
}
