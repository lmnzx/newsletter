use newsletter::{configuration::get_config, shutdown, startup::app};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use std::{net::SocketAddr, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "newsletter=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = get_config().expect("Failed to read configuration.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let app = app(pool);

    let addr: SocketAddr = format!("127.0.0.1:{}", config.application_port)
        .parse()
        .expect("Failed to parse address.");

    tracing::debug!("listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .unwrap();
}
