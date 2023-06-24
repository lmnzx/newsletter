use axum::{http::StatusCode, response::IntoResponse};

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm() -> impl IntoResponse {
    (StatusCode::OK, "everything is fine, boss 👍")
}
