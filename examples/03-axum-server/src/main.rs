use std::sync::Arc;

use axum::{
    extract::State,
    routing::get,
    Router,
};

#[derive(Clone)]
struct AppState {
    node_name: String,
}

async fn hello() -> &'static str {
    "Hello, Axum!"
}

async fn health() -> &'static str {
    "OK"
}

async fn node_info(State(state): State<Arc<AppState>>) -> String {
    format!("Node: {}", state.node_name)
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        node_name: "node-01".to_string(),
    });

    let app = Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
        .route("/node", get(node_info))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("failed to bind TCP listener");

    println!("Axum server listening on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}
