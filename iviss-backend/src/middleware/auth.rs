use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

pub async fn authorize(
    request: Request,
    next: Next,
) -> Response {
    // JWT extraction placeholder
    next.run(request).await
}
