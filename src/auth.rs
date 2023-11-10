use crate::state::{AppState, AuthSettings};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{query_as, types::Uuid, Acquire, PgPool, Postgres, Transaction};
use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Add,
};

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

#[derive(Debug)]
struct HashPasswordError {
    details: String,
}

impl HashPasswordError {
    fn new(msg: &str) -> HashPasswordError {
        HashPasswordError {
            details: msg.to_string(),
        }
    }
}

impl Display for HashPasswordError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for HashPasswordError {
    fn description(&self) -> &str {
        &self.details
    }
}

fn hash_password(password: &str) -> Result<String, HashPasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| HashPasswordError::new(&e.to_string()))?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(parsed_hash) => parsed_hash,
        Err(_) => return false,
    };

    Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn generate_access_token(
    user_id: &Uuid,
    signing_secret: &str,
    now: i64, // timestamp
    exp: i64, // timestamp
) -> Result<String, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(signing_secret.as_bytes())?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", user_id.to_string());
    claims.insert("exp", exp.to_string());
    claims.insert("iat", now.to_string());

    claims.sign_with_key(&key)
}

struct NewRefreshToken {
    id: Uuid,
}

async fn generate_refresh_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &Uuid,
    signing_secret: &str,
    exp: DateTime<Utc>,
    now: i64, // timestamp
) -> Result<String, Box<dyn Error>> {
    let jti = query_as!(
        NewRefreshToken,
        r#"
        INSERT INTO auth.refresh_tokens (user_id, expires_at)
        VALUES ($1, $2)
        RETURNING id
        "#,
        user_id,
        exp
    )
    .fetch_one(&mut **tx)
    .await?;

    let key: Hmac<Sha256> = Hmac::new_from_slice(signing_secret.as_bytes())?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", user_id.to_string());
    claims.insert("exp", exp.timestamp().to_string());
    claims.insert("iat", now.to_string());
    claims.insert("jti", jti.id.to_string());

    claims.sign_with_key(&key).map_err(|e| e.into())
}

#[derive(Serialize)]
struct AuthTokens {
    access_token: String,
    refresh_token: String,
    access_token_exp: i64,
    refresh_token_exp: i64,
}

impl AuthTokens {
    async fn generate(
        db: &mut Transaction<'_, Postgres>,
        user_id: &Uuid,
        signing_secret: &str,
        access_token_exp_mins: u16,
        refresh_token_exp_mins: u16,
    ) -> Result<Self, Box<dyn Error>> {
        let now = Utc::now();
        let access_token_exp = now.add(Duration::minutes(access_token_exp_mins as i64));
        let refresh_token_exp = now.add(Duration::minutes(refresh_token_exp_mins as i64));

        let now_ts = now.timestamp();
        let access_token_exp_ts = access_token_exp.timestamp();

        let access_token =
            generate_access_token(user_id, signing_secret, now_ts, access_token_exp_ts)?;

        let refresh_token =
            generate_refresh_token(db, user_id, signing_secret, refresh_token_exp, now_ts).await?;

        Ok(Self {
            access_token,
            refresh_token,
            access_token_exp: access_token_exp.timestamp(),
            refresh_token_exp: refresh_token_exp.timestamp(),
        })
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

    if !verify_password(&body.password, &creds.password_hash) {
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

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/tokens", post(tokens_handler))
        .with_state(state)
}
