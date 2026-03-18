mod core;
mod agent;

#[tokio::main]
async fn main() {
    core::run().await;
}
