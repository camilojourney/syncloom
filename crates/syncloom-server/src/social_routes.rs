//! Social connection route table; registration and handlers have their own owners.
use crate::{
    connection_routes::{AppRouter, register},
    social_handlers::*,
};
use axum::routing::{get, post};

pub(crate) fn routes() -> AppRouter {
    register([
        ("/v1/bluesky/sign-in", post(bluesky_sign_in)),
        ("/v1/x/sign-in", post(x_begin).get(x_wait)),
        ("/v1/threads/sign-in", post(threads_begin)),
        ("/v1/threads/complete", post(threads_complete)),
        (
            "/v1/{platform}/connection",
            get(connection).delete(sign_out),
        ),
        ("/v1/{platform}/refresh", post(refresh)),
    ])
}
