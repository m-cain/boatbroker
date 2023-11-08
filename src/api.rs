use crate::auth;
use crate::state::AppState;
use axum::Router;
use std::{error::Error, net::SocketAddr};

fn routes(state: AppState) -> Router {
    Router::new().nest("/auth", auth::routes(state))
}

pub async fn serve() -> Result<(), Box<dyn Error>> {
    let state = AppState::from_env().await;
    let routes = routes(state.clone());

    let port = state.settings.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .map_err(|e| e.into())
}
