use utoipa::OpenApi;
use iviss_backend::api_doc::ApiDoc;

fn main() {
    let openapi = ApiDoc::openapi();
    println!("{}", openapi.to_json().unwrap());
}
