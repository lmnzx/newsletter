use axum::{
    http::Request,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{self, DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::Level;
use uuid::Uuid;

use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

pub fn app(pool: PgPool, email_client: EmailClient) -> Router {
    // let email_client = Arc::new(email_client);
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_response(DefaultOnResponse::new().include_headers(true))
                        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                        .on_body_chunk(trace::DefaultOnBodyChunk::new())
                        .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR))
                        .on_eos(trace::DefaultOnEos::new().level(Level::INFO)),
                )
                .propagate_x_request_id(),
        )
        .layer(Extension(email_client))
        .with_state(pool)
}
