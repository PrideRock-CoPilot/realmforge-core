use control_service::ServiceContext;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use env-filter to allow RUST_LOG-based configuration
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let addr = std::env::var("REALMFORGE_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse::<SocketAddr>()?;

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/realmforge".to_string());

    tracing::info!(db_url = %db_url, "connecting to database");
    let state = ServiceContext::connect_and_migrate(&db_url)
        .await
        .map_err(|e| {
            format!(
                "Cannot connect to PostgreSQL (DATABASE_URL={}): {}",
                db_url, e
            )
        })?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("control-api listening on http://{addr}");

    axum::serve(listener, control_api::router(state)).await?;
    Ok(())
}
