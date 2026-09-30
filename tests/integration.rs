use netlab_app_rust::{app, AppState};
use sqlx::PgPool;
use std::env;

async fn spawn_test_server() -> String {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set (CI provides a Postgres service container)");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    let state = AppState {
        pool,
        version: "test".to_string(),
        served_by: "netlab-app-rust-test".to_string(),
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind ephemeral port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app(state)).await.unwrap();
    });

    format!("http://{}", addr)
}

#[tokio::test]
async fn root_returns_version_and_served_by() {
    let base_url = spawn_test_server().await;
    let client = reqwest::Client::new();

    let resp = client.get(format!("{base_url}/")).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["served_by"], "netlab-app-rust-test");
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn health_returns_200() {
    let base_url = spawn_test_server().await;
    let client = reqwest::Client::new();

    let resp = client.get(format!("{base_url}/health")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn visits_increments_on_each_call() {
    let base_url = spawn_test_server().await;
    let client = reqwest::Client::new();

    let first: serde_json::Value = client
        .get(format!("{base_url}/visits"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let first_count = first["visits"].as_i64().unwrap();

    let second: serde_json::Value = client
        .get(format!("{base_url}/visits"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let second_count = second["visits"].as_i64().unwrap();

    assert_eq!(second_count, first_count + 1);
}
