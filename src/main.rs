mod core;
use core::{cli, load_config};

fn main() {
    let mut config = load_config().unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);

        std::process::exit(1);
    });

    let _ = cli(&mut config);
}
