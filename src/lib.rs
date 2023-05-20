use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "everything is fine, boss 👍")
}

async fn subscribe(Form(input): Form<FormData>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!("welcome, {} - {}", input.name, input.email),
    )
}

pub fn app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
