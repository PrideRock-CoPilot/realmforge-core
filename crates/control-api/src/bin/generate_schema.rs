/// Schema generation binary.
///
/// Usage: cargo run -p control-api --bin generate-schema > openapi.json
///
/// Run this after any change to route handlers, request types, or response types.
/// Commit the updated openapi.json alongside the code change.
/// CI will fail if the committed file drifts from the generated output.
fn main() {
    use utoipa::OpenApi as _;
    let schema = control_api::ApiDoc::openapi();
    let json = schema
        .to_pretty_json()
        .expect("ApiDoc serialization cannot fail — all schema types are valid");
    println!("{json}");
}
