mod agent;
mod core;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--interactive".to_string()) || args.contains(&"-i".to_string()) {
        core::run_interactive().await;
    } else {
        core::run().await;
    }
}
