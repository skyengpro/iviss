use axum::{extract::Request, middleware::Next, response::Response};

#[allow(dead_code)]
pub async fn log_request(request: Request, next: Next) -> Response {
    tracing::info!("Request: {} {}", request.method(), request.uri());
    next.run(request).await
}
