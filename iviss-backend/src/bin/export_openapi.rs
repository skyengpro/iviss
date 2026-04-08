use iviss_backend::api_doc::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let openapi = ApiDoc::openapi();
    println!("{}", openapi.to_json().unwrap());
}
