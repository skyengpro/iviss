use axum::http::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    let x_auth_retry = HeaderName::from_static("x-auth-retry");

    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![CONTENT_TYPE, AUTHORIZATION, x_auth_retry])
}
