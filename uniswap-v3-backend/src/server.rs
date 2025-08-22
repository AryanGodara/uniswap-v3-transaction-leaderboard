use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::handlers::{health_check, leaderboard_handler};

pub async fn run_server(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/leaderboard", post(leaderboard_handler))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 Server running on http://localhost:{}", port);
    println!("🔗 API endpoint: http://localhost:{}/api/leaderboard", port);
    println!("❤️  Health check: http://localhost:{}/health", port);

    axum::serve(listener, app).await?;
    Ok(())
}
