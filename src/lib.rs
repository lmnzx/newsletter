use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "everything is fine, boss 👍")
}

pub fn app() -> Router {
    Router::new().route("/health_check", get(health_check))
}
