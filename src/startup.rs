use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::routes::{health_check, subscribe};

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool)
}
