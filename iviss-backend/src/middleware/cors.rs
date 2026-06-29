use axum::http::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    let x_auth_retry = HeaderName::from_static("x-auth-retry");

    let allow_origin = match std::env::var("ALLOWED_ORIGINS") {
        Ok(origins_str) => {
            let origins: Vec<HeaderValue> = origins_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<HeaderValue>().ok())
                .collect();

            if origins.is_empty() {
                AllowOrigin::any()
            } else {
                AllowOrigin::list(origins)
            }
        }
        Err(_) => AllowOrigin::any(),
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
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
