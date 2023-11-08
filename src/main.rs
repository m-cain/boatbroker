pub mod api;

use api::state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::from_env().await;

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api::routes(state).into_make_service())
        .await
        .unwrap();
}
