use axum::{http::StatusCode, response::IntoResponse};

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "everything is fine, boss 👍")
}
