use axum::{routing::get, Router, Extension, Json, middleware as axum_middleware};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod services;
mod models;
mod middleware;
mod utils;

use middleware::auth::{jwt_auth, api_key_auth};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("✅ Connected to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("✅ Database migrations completed");

    // Redis connection
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");

    tracing::info!("✅ Connected to Redis");

    // Create shared state
    let state = Arc::new(AppState {
        db: db_pool,
        redis: redis_client,
    });

    // API keys routes with JWT authentication middleware
    let api_keys_routes = routes::api_keys::router()
        .layer(axum_middleware::from_fn_with_state(state.clone(), jwt_auth));

    // Proxy routes with API key authentication middleware
    let proxy_routes = routes::proxy::router()
        .layer(axum_middleware::from_fn(api_key_auth));

    // Usage routes with JWT authentication
    let usage_routes = routes::usage::usage_routes()
        .with_state(state.db.clone())
        .layer(axum_middleware::from_fn_with_state(state.clone(), jwt_auth));

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .nest("/auth", routes::auth::router())
        .nest("/api-keys", api_keys_routes)
        .nest("/usage", usage_routes)
        .nest("/v1", proxy_routes)  // Uses API key auth (wbr_* keys)
        .layer(Extension(state));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Webrana AI Proxy starting on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    database: String,
}

async fn health_check_db(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<HealthStatus> {
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    Json(HealthStatus {
        status: "ok".to_string(),
        database: db_status,
    })
}
