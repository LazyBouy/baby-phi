use std::path::Path;

fn main() {
    let config_path = Path::new("config.toml");
    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            match toml::from_str::<toml::value::Table>(&content) {
                Ok(_) => println!("✓ TOML syntax is valid"),
                Err(e) => println!("✗ TOML parse error: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
}
