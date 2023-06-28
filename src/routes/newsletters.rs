use anyhow::Context;
use axum::{
    extract::{Json, State},
    http::{header::HeaderMap, StatusCode},
    response::IntoResponse,
    Extension,
};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sha3::Digest;
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, email_client::EmailClient};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    struct Row {
        email: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .filter_map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Some(ConfirmedSubscriber { email }),
            Err(error) => {
                tracing::error!(
                    "A confirmed subscriber is using an invalid email address.\n{}.",
                    error
                );
                None
            }
        })
        .collect();
    Ok(confirmed_subscribers)
}

#[tracing::instrument(
name = "Publish a newsletter issue",
skip(payload, pool, email_client), fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Extension(email_client): Extension<EmailClient>,
    Json(payload): Json<BodyData>,
) -> impl IntoResponse {
    let credentials = match basic_auth(&headers) {
        Ok(credentials) => credentials,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = match validate_credentials(credentials, &pool).await {
        Ok(user_id) => user_id,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&pool).await.unwrap();
    for subscriber in subscribers {
        if email_client
            .send_email(
                subscriber.email,
                &payload.title,
                &payload.content.text,
                &payload.content.html,
            )
            .await
            .is_err()
        {
            tracing::error!("failed to send welcome email");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    StatusCode::OK
}

fn basic_auth(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization header was not a valid UTF-8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context(" The authorization scheme was not 'Basic'.")?;
    let decoded_byters = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_byters)
        .context("The decoded credentials were not valid UTF-8.")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("The decoded credentials did not contain a username."))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("The decoded credentials did not contain a password."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, sqlx::Error> {
    let password_hash = sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    let password_hash = format!("{:x}", password_hash);
    let user_id: Option<_> = sqlx::query!(
        r#"
        SELECT user_id
        FROM users
        WHERE username = $1 AND password_hash = $2
        "#,
        credentials.username,
        password_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;

    user_id
        .map(|row| row.user_id)
        .ok_or_else(|| sqlx::Error::RowNotFound)
}
