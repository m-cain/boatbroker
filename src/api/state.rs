use axum::extract::FromRef;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

#[derive(Clone)]
struct PostgresSettings {
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
struct Settings {
    postgres: PostgresSettings,
}

impl Settings {
    pub fn from_env() -> Self {
        Self {
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
    db_pool: PgPool,
    settings: Settings,
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
