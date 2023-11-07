pub mod api;

#[tokio::main]
async fn main() {
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api::router().into_make_service())
        .await
        .unwrap();
}
