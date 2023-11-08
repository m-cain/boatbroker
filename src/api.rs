pub mod auth;
pub mod state;

use axum::Router;

use self::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new().nest("/auth", auth::routes(state))
}
