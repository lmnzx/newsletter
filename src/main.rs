use newsletter::{configuration::get_config, email_client::EmailClient, shutdown, startup::app};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use std::{net::SocketAddr, time::Duration};

// todo: securing the api
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

    let config = get_config().expect("failed to read configuration.");

    let base_url = config.application.base_url.clone();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(config.database.with_db());

    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );

    let app = app(pool, email_client, base_url);

    let addr: SocketAddr = format!("{}:{}", config.application.host, config.application.port)
        .parse()
        .expect("failed to parse address.");

    tracing::debug!("listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .unwrap();
}
