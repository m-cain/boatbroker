use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Serialize;
use sha2::Sha256;
use sqlx::{query_as, types::Uuid, Postgres, Transaction};
use std::{collections::BTreeMap, error::Error, ops::Add};

pub fn generate_access_token(
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

struct CreateRefreshTokenRow {
    id: Uuid,
}

pub async fn generate_refresh_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &Uuid,
    signing_secret: &str,
    exp: DateTime<Utc>,
    now: i64, // timestamp
) -> Result<String, Box<dyn Error>> {
    let jti = query_as!(
        CreateRefreshTokenRow,
        r#"
      insert into auth.refresh_tokens (user_id, expires_at)
      values ($1, $2)
      returning id
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
pub struct AuthTokens {
    access_token: String,
    refresh_token: String,
    access_token_exp: i64,
    refresh_token_exp: i64,
}

impl AuthTokens {
    pub async fn generate(
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
