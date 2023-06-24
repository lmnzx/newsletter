use axum::{http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm() -> impl IntoResponse {
    (StatusCode::OK, "everything is fine, boss 👍")
}
