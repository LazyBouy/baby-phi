mod agent;
mod core;

#[tokio::main]
async fn main() {
    core::run().await;
}
