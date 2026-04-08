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
) -> std::io::Result<()> {
    if src.ends_with(".git") {
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let file_name = render_file(entry.file_name().to_str().unwrap_or_default(), context);

        let dest_path = dst.join(file_name.clone());
        if file_type.is_dir() {
            render_template_dir(&entry.path(), &dest_path, context, pb)?;
        } else {
            if file_name == "schema.json" {
                continue;
            }

            let file_content = fs::read_to_string(entry.path())?;
            let rendered_content = render_file(&file_content, context);
            fs::write(&dest_path, rendered_content)?;

            pb.inc(1);
        }
    }

    Ok(())
}
