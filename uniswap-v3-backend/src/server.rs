use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::handlers::{health_check, leaderboard_handler};
use crate::config::Config;

pub async fn run_server(port: u16) -> Result<()> {
    let config = Config::from_env()?;
    
    let cors = CorsLayer::new()
        .allow_origin(config.allowed_origins.iter().map(|origin| origin.parse().unwrap()).collect::<Vec<_>>())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/leaderboard", post(leaderboard_handler))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, port)).await?;
    println!("🚀 Server running on http://localhost:{}", port);
    println!("🔗 API endpoint: http://localhost:{}/api/leaderboard", port);
    println!("❤️  Health check: http://localhost:{}/health", port);

    axum::serve(listener, app).await?;
    Ok(())
}
