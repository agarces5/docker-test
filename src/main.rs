use std::collections::HashMap;

use axum::Router;
use axum::extract::Query;
use axum::routing::get;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

async fn hello(Query(params): Query<HashMap<String, String>>) -> String {
    let name = params
        .get("name")
        .unwrap_or(&"World".to_string())
        .to_string();
    format!("Hello {name}!")
}
#[tokio::main]
async fn main() {
    // initialize env vars
    dotenvy::dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .compact()
        .init();

    //build app
    let app = Router::new().route("/", get(hello)).layer((
        tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(tower_http::trace::DefaultMakeSpan::new())
            .on_response(tower_http::trace::DefaultOnResponse::new()),
        tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(5)),
    ));

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
