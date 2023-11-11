use super::{password, tokens::AuthTokens};
use crate::state::{AppState, AuthSettings};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use sqlx::{query_as, types::Uuid, Acquire, PgPool};

const CHALLENGE_HEADER: (&str, &str) = ("www-authenticate", "Bearer");

struct UnauthenticatedError;

impl IntoResponse for UnauthenticatedError {
    fn into_response(self) -> Response {
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
    fn into_response(self) -> Response {
        (StatusCode::FORBIDDEN, [CHALLENGE_HEADER], "Unauthorized.").into_response()
    }
}

#[derive(Deserialize)]
struct TokensRequest {
    email: String,
    password: String,
}

#[derive(sqlx::FromRow)]
struct CredentialsRow {
    password_hash: String,
    id: Uuid,
}

async fn tokens_handler(
    State(pg_pool): State<PgPool>,
    State(auth_settings): State<AuthSettings>,
    Json(body): Json<TokensRequest>,
) -> impl IntoResponse {
    let mut conn = pg_pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    let creds = query_as!(
        CredentialsRow,
        r#"
        SELECT password_hash, id
        FROM auth.users
        WHERE email = $1
        "#,
        body.email
    )
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    let creds = match creds {
        Some(creds) => creds,
        None => return UnauthorizedError.into_response(),
    };

    if !password::verify(&body.password, &creds.password_hash) {
        return UnauthorizedError.into_response();
    }

    let tokens = AuthTokens::generate(
        &mut tx,
        &creds.id,
        &auth_settings.signing_secret,
        auth_settings.access_token_exp_minutes,
        auth_settings.refresh_token_exp_minutes,
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    Json(tokens).into_response()
}

pub fn router<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/tokens", post(tokens_handler))
        .with_state(state)
}
