use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Router};
use serde::Deserialize;
use sqlx::PgPool;

const CHALLENGE_HEADER: (&str, &str) = ("www-authenticate", "Bearer");

struct UnauthenticatedError;

impl IntoResponse for UnauthenticatedError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::UNAUTHORIZED,
            [CHALLENGE_HEADER],
            "Unauthenticated.",
        )
            .into_response()
    }
}

struct UnauthorizedError;

impl IntoResponse for UnauthorizedError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::FORBIDDEN, [CHALLENGE_HEADER], "Unauthorized.").into_response()
    }
}

#[derive(Deserialize)]
pub struct TokensRequest {
    username: String,
    password: String,
}

async fn tokens_handler(State(pg_pool): State<PgPool>) {
    todo!()
}

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/tokens", post(tokens_handler))
        .with_state(state)
}
