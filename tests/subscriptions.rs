// test
#[cfg(test)]
mod tests {
    use newsletter::{
        configuration::{get_config, DatabaseSettings},
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
        let mut connection = PgConnection::connect(&config.connection_string_without_db())
            .await
            .expect("Failed to connect to Postgres");

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create database.");

        // Migrate database
        let pool = PgPool::connect(&config.connection_string())
            .await
            .expect("Failed to connect to Postgres.");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate the database");

        pool
    }

    #[tokio::test]
    async fn subscriptions_valid_data_test() {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let mut config = get_config().expect("Failed to read configuration.");

        config.database.database_name = Uuid::new_v4().to_string();

        let pool = db_config(&config.database).await;

        let app = app(pool);

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
    async fn subscriptions_invalid_data_test() {
        let test_cases = vec![
            ("name=le%20mon", "missing the name"),
            ("email=leemon%40gmail.com", "missing the name"),
            ("", "missing both name and email"),
        ];

        for (invaild_body, error_message) in test_cases {
            let mut config = get_config().expect("Failed to read configuration.");

            config.database.database_name = Uuid::new_v4().to_string();

            let pool = db_config(&config.database).await;

            let app = app(pool);

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
