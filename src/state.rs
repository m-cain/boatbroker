use axum::extract::FromRef;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

#[derive(Clone)]
pub struct PostgresSettings {
    dsn: String,
    max_connections: u32,
    min_connections: u32,
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
    pub port: u16,
    pub postgres: PostgresSettings,
}

impl Settings {
    pub fn from_env() -> Self {
        Self {
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
