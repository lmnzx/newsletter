use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
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
    let new_subscriber = match input.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(&pool, &new_subscriber).await {
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
    skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
