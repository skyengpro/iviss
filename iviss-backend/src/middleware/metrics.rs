use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub async fn track_metrics(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();

    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let response = next.run(req).await;

    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "iviss_http_requests_total",
        "method" => method.to_string(),
        "path" => path.clone(),
        "status" => status
    )
    .increment(1);

    metrics::histogram!(
        "iviss_http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path
    )
    .record(elapsed);

    response
}
