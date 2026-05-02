use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("REALMFORGE_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse::<SocketAddr>()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("realmforge-core api listening on http://{addr}");
    axum::serve(listener, rf_api::router()).await?;
    Ok(())
}
