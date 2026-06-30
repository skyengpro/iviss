use axum::http::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn cors_layer(cors_allowed_origins: &[String]) -> CorsLayer {
    let x_auth_retry = HeaderName::from_static("x-auth-retry");

    let origins = cors_allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin)
                .expect("ALLOWED_ORIGINS must contain valid HTTP header values")
        })
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![CONTENT_TYPE, AUTHORIZATION, x_auth_retry])
}
