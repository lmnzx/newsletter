use newsletter::{configuration::get_config, startup::app};
use sqlx::postgres::PgPoolOptions;

use std::{net::SocketAddr, time::Duration};

#[tokio::main]
async fn main() {
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
    println!("listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
