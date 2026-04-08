use handlebars::Handlebars;
use indicatif::ProgressBar;
use serde_json::Value;
use std::fs;
use std::path::Path;

fn render_file(template: &str, context: &Value) -> String {
    let reg = Handlebars::new();

    reg.render_template(template, context)
        .unwrap_or_else(|err| {
            eprintln!("Template rendering error: {}", err);
            template.to_string()
        })
}

pub fn render_template_dir(
    src: &Path,
    dst: &Path,
    context: &Value,
    pb: &ProgressBar,
    excluded_root: Option<&Path>,
) -> std::io::Result<()> {
    if src.ends_with(".git") {
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();

        if excluded_root.is_some_and(|excluded_root| entry_path.starts_with(excluded_root)) {
            continue;
        }

        let file_type = entry.file_type()?;

        let file_name = render_file(entry.file_name().to_str().unwrap_or_default(), context);

        let dest_path = dst.join(file_name.clone());
        if file_type.is_dir() {
            render_template_dir(&entry_path, &dest_path, context, pb, excluded_root)?;
        } else {
            if file_name == "schema.json" {
                continue;
            }

            let file_content = fs::read_to_string(entry_path)?;
            let rendered_content = render_file(&file_content, context);
            fs::write(&dest_path, rendered_content)?;

            pb.inc(1);
        }
    }

    Ok(())
}
