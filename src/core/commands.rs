use crate::core::count_dir_files_except;

use super::{
    Config, RemoteSchematic, Schematic, SchematicDetail, handle_inputs, read_schema,
    render_template_dir, save_config,
};
use indicatif::ProgressBar;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn show_schematics(
    schematics: &BTreeMap<String, Schematic>,
) -> Result<(), Box<dyn std::error::Error>> {
    let schematic_details: Vec<SchematicDetail> = schematics
        .iter()
        .map(|(name, schematic)| SchematicDetail::new(name, schematic))
        .collect();

    if let Err(err) = SchematicDetail::print_table(schematic_details) {
        return Err(format!("Failed to display schematics: {}", err).into());
    }

    Ok(())
}

pub fn add_schematic(
    config: &mut Config,
    name: String,
    schematic: Schematic,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.schematics.contains_key(&name) {
        return Err("Duplicate schematic name".into());
    }

    config.schematics.insert(name, schematic);

    if let Err(err) = save_config(config) {
        return Err(format!("Failed to save config: {}", err).into());
    }

    println!("Schematic added successfully.");

    Ok(())
}

pub fn remove_schematic(config: &mut Config, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if config.schematics.remove(name).is_none() {
        return Err(format!("No schematic found with name '{}'.", name).into());
    }

    if let Err(err) = save_config(config) {
        return Err(format!("Failed to save config: {}", err).into());
    }

    println!("Schematic '{}' removed successfully.", name);

    return Ok(());
}

fn render_schematic(
    schema_json: &Value,
    src: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let inputs = if schema_json.is_null() {
        Value::Object(Map::new())
    } else {
        match handle_inputs(schema_json) {
            Ok(data) => data,
            Err(e) => match e {
                super::InputError::DialoguerError(err) => {
                    return Err(format!("Dialoguer error: {}", err).into());
                }
                super::InputError::ValidationErrors(err) => {
                    let errs = err
                        .into_iter()
                        .map(|e| format!("- {}", e))
                        .collect::<Vec<String>>()
                        .join("\n");

                    return Err(format!("Validation errors: {}", errs).into());
                }
            },
        }
    };

    if let Err(err) = fs::create_dir_all(destination) {
        return Err(format!("Failed to create destination directory {}", err).into());
    }

    let src = fs::canonicalize(src)?;
    let dest = fs::canonicalize(destination)?;
    let excluded_root = dest
        .starts_with(&src)
        .then_some(dest.as_path())
        .filter(|path| *path != src.as_path());

    let total_files = count_dir_files_except(&src, excluded_root).unwrap_or(0);

    let pb = ProgressBar::new(total_files);

    if let Err(err) = render_template_dir(&src, &dest, &inputs, &pb, excluded_root) {
        return Err(format!("Failed to copy files: {}", err).into());
    }

    pb.finish_with_message("Schematic generation completed.");

    Ok(())
}

fn remote_render_schematic(
    remote: &RemoteSchematic,
    tmp_path: &PathBuf,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = std::process::Command::new("git");
    command.arg("clone");

    if let Some(branch) = &remote.branch {
        command.arg("--branch").arg(branch);
    }

    let status = command
        .arg(remote.url.clone())
        .arg(tmp_path.clone())
        .status()?;

    if !status.success() {
        return Err("Git clone failed".into());
    }

    let path = Path::join(tmp_path.as_path(), Path::new("schema.json"));

    let schema_json = read_schema(&path);

    render_schematic(
        &schema_json,
        tmp_path.to_str().as_ref().unwrap(),
        destination,
    )?;

    Ok(())
}

pub fn generate_schematic(
    schematics: &BTreeMap<String, Schematic>,
    name: &str,
    destination: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let schematic = schematics.get(name);
    if schematic.is_none() {
        return Err(format!("No schematic found with name '{}'.", name).into());
    }
    let schematic = schematic.unwrap();

    let default_destination = format!("./{}", name);
    let destination = destination.as_deref().unwrap_or(&default_destination);

    println!("Generating schematic: {} to {}", name, destination);

    match schematic {
        Schematic::Simple(path) => {
            let schema_path = Path::join(Path::new(&path), Path::new("schema.json"));

            let schema_json = read_schema(&schema_path);

            return render_schematic(&schema_json, &path, destination);
        }
        Schematic::Local(local) => {
            let path = &Path::join(Path::new(&local.path), Path::new("schema.json"));

            let schema_json = read_schema(path);

            return render_schematic(&schema_json, &local.path, destination);
        }
        Schematic::Remote(remote) => {
            let unique_suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            let tmp_path = std::env::temp_dir().join(format!(
                "schematics_tmp_{}_{}",
                std::process::id(),
                unique_suffix
            ));

            let render_result = remote_render_schematic(remote, &tmp_path, destination);

            if tmp_path.exists() {
                if let Err(err) = fs::remove_dir_all(&tmp_path) {
                    eprintln!("Failed to remove temporary directory: {}", err);
                }
            }

            return render_result;
        }
    }
}
