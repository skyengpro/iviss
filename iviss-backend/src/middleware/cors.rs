use tower_http::cors::CorsLayer;

#[allow(dead_code)]
pub fn cors_layer() -> CorsLayer {
    CorsLayer::permissive()
}
