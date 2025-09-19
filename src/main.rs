use std::collections::HashMap;

use axum::Router;
use axum::extract::Query;
use axum::routing::get;

async fn hello(Query(params): Query<HashMap<String, String>>) -> String {
    tracing::debug!("{params:?}");
    let name = params
        .get("name")
        .unwrap_or(&"World".to_string())
        .to_string();
    format!("Hello {name}!")
}
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // initialize env vars
    dotenvy::dotenv().ok();

    //build app
    let app = Router::new().route("/", get(hello));

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
