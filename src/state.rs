use axum::extract::FromRef;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

#[derive(Clone)]
pub struct AuthSettings {
    pub signing_secret: String,
    pub access_token_exp_minutes: u16,
    pub refresh_token_exp_minutes: u16,
}

impl AuthSettings {
    fn from_env() -> Self {
        Self {
            signing_secret: env::var("AUTH_SIGNING_SECRET")
                .expect("AUTH_SIGNING_SECRET must be set"),
            access_token_exp_minutes: env::var("AUTH_ACCESS_TOKEN_EXP_MINUTES")
                .unwrap_or("30".into())
                .parse()
                .expect("AUTH_ACCESS_TOKEN_EXP_MINUTES must be an integer"),
            refresh_token_exp_minutes: env::var("AUTH_REFRESH_TOKEN_EXP_MINUTES")
                .unwrap_or((24 * 60 * 90).to_string().into())
                .parse()
                .expect("AUTH_REFRESH_TOKEN_EXP_MINUTES must be an integer"),
        }
    }
}

impl FromRef<AppState> for AuthSettings {
    fn from_ref(state: &AppState) -> Self {
        state.settings.auth.clone()
    }
}

#[derive(Clone)]
pub struct PostgresSettings {
    pub dsn: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl PostgresSettings {
    fn from_env() -> Self {
        Self {
            dsn: env::var("POSTGRES_DSN")
                .unwrap_or("postgres://postgres:postgres@localhost:5432".into()),
            max_connections: env::var("POSTGRES_MAX_CONNECTIONS")
                .unwrap_or("5".into())
                .parse()
                .expect("POSTGRES_MAX_CONNECTIONS must be an integer"),
            min_connections: env::var("POSTGRES_MIN_CONNECTIONS")
                .unwrap_or("1".into())
                .parse()
                .expect("POSTGRES_MIN_CONNECTIONS must be an integer"),
        }
    }
}

#[derive(Clone)]
pub struct Settings {
    pub auth: AuthSettings,
    pub port: u16,
    pub postgres: PostgresSettings,
}

impl Settings {
    pub fn from_env() -> Self {
        Self {
            auth: AuthSettings::from_env(),
            port: env::var("PORT")
                .unwrap_or("3000".into())
                .parse()
                .expect("PORT must be an integer"),
            postgres: PostgresSettings::from_env(),
        }
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub settings: Settings,
}

impl AppState {
    pub async fn from_env() -> Self {
        let settings = Settings::from_env();
        let db_pool = PgPoolOptions::new()
            .max_connections(settings.postgres.max_connections)
            .min_connections(settings.postgres.min_connections)
            .connect(&settings.postgres.dsn)
            .await
            .expect("Failed to connect to Postgres");

        Self { db_pool, settings }
    }
}
