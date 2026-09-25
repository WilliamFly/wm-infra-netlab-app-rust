use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;

struct AppState {
    pool: PgPool,
    version: String,
    served_by: String,
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@10.0.3.20:5432/netlab_app)");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");
    let served_by = env::var("SERVED_BY").unwrap_or_else(|_| "netlab-app-rust".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database — is the DB VM reachable and running?");

    // Runs any migration in ./migrations not yet applied. Safe to run on
    // every start — sqlx tracks what's already been applied in its own
    // _sqlx_migrations table.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(AppState {
        pool,
        version: env!("CARGO_PKG_VERSION").to_string(),
        served_by,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/visits", get(visits))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");
    axum::serve(listener, app).await.expect("server error");
}

async fn root(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "version": state.version,
        "served_by": state.served_by,
    }))
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn visits(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("INSERT INTO visits DEFAULT VALUES")
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM visits")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "visits": count })))
}
