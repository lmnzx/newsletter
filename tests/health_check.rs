// test
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use newsletter::{configuration::get_config, startup::app};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_check_test() {
        let config = get_config().expect("failed to read configuration.");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.database.connection_string())
            .await
            .expect("failed to connect to Postgres.");

        let app = app(pool);

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
