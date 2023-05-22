use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use tower_http::{trace, trace::TraceLayer};
use tracing::Level;

use crate::routes::{health_check, subscribe};

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .on_body_chunk(trace::DefaultOnBodyChunk::new())
                .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR))
                .on_eos(trace::DefaultOnEos::new().level(Level::INFO)),
        )
        .with_state(pool)
}
