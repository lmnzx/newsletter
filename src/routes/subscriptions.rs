use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(input): Form<FormData>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"
      INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
      "#,
        Uuid::new_v4(),
        input.email,
        input.name,
        Utc::now()
    )
    .execute(&pool)
    .await
    .expect("Database error");
    (
        StatusCode::OK,
        format!("welcome, {} - {}", input.name, input.email),
    )
}
