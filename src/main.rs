pub mod api;

#[tokio::main]
async fn main() {
    api::serve().await.unwrap();
}
