use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub version: String,
    pub served_by: String,
}

#[derive(Serialize)]
struct RootResponse {
    version: String,
    served_by: String,
}

#[derive(Serialize)]
struct VisitsResponse {
    visits: i64,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/visits", get(visits))
        .with_state(Arc::new(state))
}

async fn root(State(state): State<Arc<AppState>>) -> Json<RootResponse> {
    Json(RootResponse {
        version: state.version.clone(),
        served_by: state.served_by.clone(),
    })
}

async fn health() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

async fn visits(State(state): State<Arc<AppState>>) -> Json<VisitsResponse> {
    sqlx::query("INSERT INTO visits DEFAULT VALUES")
        .execute(&state.pool)
        .await
        .expect("insert failed");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM visits")
        .fetch_one(&state.pool)
        .await
        .expect("count query failed");

    Json(VisitsResponse { visits: count })
}
