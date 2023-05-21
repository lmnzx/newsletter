// test
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use newsletter::{configuration::get_config, startup::app};

    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    #[tokio::test]
    async fn subscriptions_valid_data_test() {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let config = get_config().expect("Failed to read configuration.");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.database.connection_string())
            .await
            .expect("Failed to connect to Postgres.");

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
            let config = get_config().expect("Failed to read configuration.");

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&config.database.connection_string())
                .await
                .expect("Failed to connect to Postgres.");

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
