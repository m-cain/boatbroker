pub mod api;
pub mod auth;
pub mod state;

#[tokio::main]
async fn main() {
    api::serve().await.unwrap();
}
