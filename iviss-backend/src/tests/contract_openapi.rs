use crate::api_doc::ApiDoc;
use utoipa::OpenApi;

const GOLDEN_OPENAPI: &str = include_str!("../../../frontend/openapi.json");

#[test]
fn generated_openapi_matches_normalized_golden_file() {
    let expected: serde_json::Value = serde_json::from_str(GOLDEN_OPENAPI)
        .expect("frontend OpenAPI golden file must be valid JSON");
    let actual: serde_json::Value = serde_json::from_str(
        &ApiDoc::openapi()
            .to_json()
            .expect("generated OpenAPI must serialize to JSON"),
    )
    .expect("generated OpenAPI must be valid JSON");

    assert_eq!(actual, expected, "generated OpenAPI contract changed");
}
