use crate::{auth, state::AppState};
use axum::Router;
use std::{future::Future, net::SocketAddr};

fn routes(state: AppState) -> Router {
    Router::new().nest("/auth", auth::routes(state))
}

async fn shutdown(sig: impl Future<Output = ()>, state: AppState) {
    sig.await;
    println!("Shutting down...");
    state.db_pool.close().await;
}

pub async fn serve<S: Future<Output = ()>>(sig: S) {
    let state = AppState::from_env().await;
    let routes = routes(state.clone());

    let port = state.settings.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let graceful = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(shutdown(sig, state))
        .await;

    if let Err(e) = graceful {
        eprintln!("server error: {}", e);
    }
}
