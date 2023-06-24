use axum::Extension;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;

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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(30)
        .collect()
}

#[tracing::instrument(
  name = "Adding a new subscriber",
  skip(pool,email_client, input),
  fields(
    email = %input.email,
    name = %input.name
  )
)]
pub async fn subscribe(
    State(pool): State<PgPool>,
    Extension(email_client): Extension<EmailClient>,
    Extension(base_url): Extension<String>,
    Form(input): Form<FormData>,
) -> impl IntoResponse {
    let new_subscriber = match input.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let subscription_token = generate_subscription_token();
    let subscriber_id: Uuid = match insert_subscriber(&pool, &new_subscriber).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("failed to execute query: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    if store_token(&pool, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if send_confirmation_email(
        &email_client,
        &new_subscriber,
        &base_url,
        &subscription_token,
    )
    .await
    .is_err()
    {
        tracing::error!("failed to send welcome email");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    subscriber_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscriber_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email.clone(),
            "Welcome!",
            &html_body,
            &plain_body,
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
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
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, pool)
)]
pub async fn store_token(
    pool: &PgPool,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
