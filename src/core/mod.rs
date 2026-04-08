mod config;
pub use config::*;

mod cli;
pub use cli::*;

mod commands;
pub use commands::*;

mod schematic;
pub use schematic::*;

mod input;
pub use input::*;

mod render;
pub use render::*;

use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn read_schema(path: &Path) -> Value {
    if fs::exists(path).unwrap_or(false) == false {
        return Value::Null;
    }

    let schema_content = fs::read_to_string(path).expect("Unable to read schema file");
    let schema_json: Value = serde_json::from_str(&schema_content).expect("Invalid JSON schema");

    schema_json
}

pub fn count_dir_files_except(src: &Path, excluded_root: Option<&Path>) -> std::io::Result<u64> {
    let mut count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();

        if excluded_root.is_some_and(|excluded_root| entry_path.starts_with(excluded_root)) {
            continue;
        }

        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            count += count_dir_files_except(&entry_path, excluded_root)?;
        } else {
            count += 1;
        }
    }

    Ok(count)
}
