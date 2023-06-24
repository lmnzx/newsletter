// test
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use newsletter::{configuration::get_config, email_client::EmailClient, startup::app};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_check_test() {
        let config = get_config().expect("failed to read configuration.");

        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let base_url = config.application.base_url.clone();

        let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.authorization_token,
            std::time::Duration::from_millis(200),
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(config.database.without_db())
            .await
            .expect("failed to connect to Postgres.");

        let app = app(pool, email_client, base_url);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health_check")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
