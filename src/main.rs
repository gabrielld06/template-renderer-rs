mod core;
use core::{cli, load_config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config().map_err(|err| format!("Failed to load config: {}", err))?;

    cli(&mut config)
}
