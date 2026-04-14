use iviss_backend::api_doc::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let openapi = ApiDoc::openapi();
    let json = openapi
        .to_pretty_json()
        .expect("Failed to serialize OpenAPI spec");
    println!("{}", json);
}
