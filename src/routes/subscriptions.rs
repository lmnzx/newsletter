use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
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
    request_id = %Uuid::new_v4(),
    email = %input.email,
    name = %input.name
  )
)]
pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(input): Form<FormData>,
) -> impl IntoResponse {
    let query_span = tracing::info_span!("saving new subscriber details in the database");
    match sqlx::query!(
        r#"
      INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
      "#,
        Uuid::new_v4(),
        input.email,
        input.name,
        Utc::now()
    )
    .execute(&pool)
    .instrument(query_span)
    .await
    {
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
