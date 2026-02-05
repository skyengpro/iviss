use axum::{extract::Request, middleware::Next, response::Response};

pub async fn log_request(request: Request, next: Next) -> Response {
    tracing::info!("Request: {} {}", request.method(), request.uri());
    next.run(request).await
}
