use rust_api_gateway::{config::AppConfig, router};
use std::net::SocketAddr;
use tower_sessions::MemoryStore;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load Config first to get the log level
    let config = AppConfig::new()?;

    // Initialize Tracing with configured level and format
    let env_filter = tracing_subscriber::EnvFilter::new(format!(
        "rust_api_gateway={},tower_http={}",
        config.server.log_level, config.server.log_level
    ));

    if config.server.log_format == "json" {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    // In production, use Redis or a real database.
    let session_store = MemoryStore::default();

    let app = router::app(config.clone(), session_store);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
