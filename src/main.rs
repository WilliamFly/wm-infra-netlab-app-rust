use netlab_app_rust::{app, AppState};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");
    let served_by = env::var("SERVED_BY").unwrap_or_else(|_| "netlab-app-rust".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    let state = AppState {
        pool,
        version: env!("CARGO_PKG_VERSION").to_string(),
        served_by,
    };

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");

    println!("{} listening on port {}", state.served_by, port);

    axum::serve(listener, app(state)).await.expect("server error");
}
