pub mod api;
pub mod auth;
pub mod state;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };

    let termination = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install termination signal handler")
            .recv()
            .await
            .expect("failed to receive termination signal")
    };

    tokio::select! {
        _ = ctrl_c => println!("CTRL+C received"),
        _ = termination => println!("Termination signal received"),
    }
}

#[tokio::main]
async fn main() {
    let sig = shutdown_signal();
    api::serve(sig).await;
}
