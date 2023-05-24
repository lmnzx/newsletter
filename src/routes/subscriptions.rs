use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
  name = "Adding a new subscriber",
  skip(pool, input),
  fields(
    email = %input.email,
    name = %input.name
  )
)]
pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(input): Form<FormData>,
) -> impl IntoResponse {
    match insert_subscriber(&pool, &input).await {
        Ok(_) => {
            tracing::info!("new subscriber saved");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, input)
)]
pub async fn insert_subscriber(pool: &PgPool, input: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        input.email,
        input.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
