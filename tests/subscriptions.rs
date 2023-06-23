// test
#[cfg(test)]
mod tests {
    use newsletter::{
        configuration::{get_config, DatabaseSettings},
        email_client::EmailClient,
        startup::app,
    };

    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use tower::ServiceExt;
    use uuid::Uuid;

    pub async fn db_config(config: &DatabaseSettings) -> PgPool {
        // Create database
        let mut connection = PgConnection::connect_with(&config.without_db())
            .await
            .expect("failed to connect to Postgres");

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("failed to create database.");

        // Migrate database
        let pool = PgPool::connect_with(config.with_db())
            .await
            .expect("failed to connect to Postgres.");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to migrate the database");

        pool
    }

    #[tokio::test]
    async fn subscriptions_valid_data_test() {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let mut config = get_config().expect("failed to read configuration.");

        config.database.database_name = format!("test-{}", Uuid::new_v4().to_string());

        let pool = db_config(&config.database).await;

        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.authorization_token,
            std::time::Duration::from_millis(200),
        );

        let app = app(pool, email_client);

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/subscriptions")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.to_string(),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
        let test_cases = vec![
            ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
            ("name=Ursula&email=", "empty email"),
            ("name=Ursula&email=definitely-not-an-email", "invalid email"),
        ];

        for (body, description) in test_cases {
            let mut config = get_config().expect("failed to read configuration.");

            config.database.database_name = format!("test-{}", Uuid::new_v4().to_string());

            let pool = db_config(&config.database).await;

            let sender_email = config
                .email_client
                .sender()
                .expect("Invalid sender email address.");

            let email_client = EmailClient::new(
                config.email_client.base_url,
                sender_email,
                config.email_client.authorization_token,
                std::time::Duration::from_millis(200),
            );

            let app = app(pool, email_client);

            let response = app
                .oneshot(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/subscriptions")
                        .header(
                            http::header::CONTENT_TYPE,
                            mime::APPLICATION_WWW_FORM_URLENCODED.to_string(),
                        )
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "{}",
                description
            );
        }
    }

    #[tokio::test]
    async fn subscriptions_invalid_data_test() {
        let test_cases = vec![
            ("name=le%20mon", "missing the name"),
            ("email=leemon%40gmail.com", "missing the name"),
            ("", "missing both name and email"),
        ];

        for (invaild_body, error_message) in test_cases {
            let mut config = get_config().expect("failed to read configuration.");

            config.database.database_name = format!("test-{}", Uuid::new_v4().to_string());

            let pool = db_config(&config.database).await;

            let sender_email = config
                .email_client
                .sender()
                .expect("Invalid sender email address.");

            let email_client = EmailClient::new(
                config.email_client.base_url,
                sender_email,
                config.email_client.authorization_token,
                std::time::Duration::from_millis(200),
            );

            let app = app(pool, email_client);

            let response = app
                .oneshot(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/subscriptions")
                        .header(
                            http::header::CONTENT_TYPE,
                            mime::APPLICATION_WWW_FORM_URLENCODED.to_string(),
                        )
                        .body(Body::from(invaild_body))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "{}",
                error_message
            );
        }
    }
}

/*
--to delete all test databases--
SELECT 'DROP DATABASE ' || quote_ident(datname) || ';'
FROM pg_database
WHERE datname LIKE 'test%' AND datistemplate=false

\gexec
*/
