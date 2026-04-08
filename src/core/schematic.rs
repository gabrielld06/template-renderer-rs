use cli_table::{
    Color, Table, WithTitle,
    format::{Align, Border, HorizontalLine, Justify, Separator, VerticalLine},
    print_stdout,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Schematic {
    Simple(String),
    Local(LocalSchematic),
    Remote(RemoteSchematic),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalSchematic {
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoteSchematic {
    pub url: String,
    pub description: Option<String>,
    pub branch: Option<String>,
}

impl Schematic {
    pub fn new_simple(path: String) -> Self {
        Schematic::Simple(path)
    }

    pub fn new_local(path: String, description: Option<String>) -> Self {
        Schematic::Local(LocalSchematic { path, description })
    }

    pub fn new_remote(url: String, branch: Option<String>, description: Option<String>) -> Self {
        Schematic::Remote(RemoteSchematic {
            url,
            branch,
            description,
        })
    }
}

impl Display for Schematic {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Schematic::Simple(path) => write!(f, "{}", path),
            Schematic::Local(local) => write!(
                f,
                "path={}, description={}",
                local.path,
                local.description.as_deref().unwrap_or("No description")
            ),
            Schematic::Remote(remote) => write!(
                f,
                "url={}, branch={}, description={}",
                remote.url,
                remote.branch.as_deref().unwrap_or("default"),
                remote.description.as_deref().unwrap_or("No description")
            ),
        }
    }
}

#[derive(Debug, Table)]
pub struct SchematicDetail {
    #[table(
        title = "Name",
        justify = "Justify::Left",
        align = "Align::Top",
        color = "Color::Blue",
        bold
    )]
    pub name: String,
    #[table(
        title = "Description",
        justify = "Justify::Right",
        align = "Align::Top",
        bold
    )]
    pub description: String,
}

impl SchematicDetail {
    pub fn new(name: &str, schematic: &Schematic) -> Self {
        let description = match schematic {
            Schematic::Simple(_) => "No description".to_string(),
            Schematic::Local(local) => local
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string()),
            Schematic::Remote(remote) => remote
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string()),
        };
        SchematicDetail {
            name: name.to_string(),
            description,
        }
    }

    pub fn print_table(details: Vec<SchematicDetail>) -> std::io::Result<()> {
        if details.is_empty() {
            println!("No schematics available.");
            return Ok(());
        }

        print_stdout(
            details
                .with_title()
                .table()
                .border(
                    Border::builder()
                        .top(HorizontalLine::new('┌', '┐', '┬', '─'))
                        .bottom(HorizontalLine::new('└', '┘', '┴', '─'))
                        .left(VerticalLine::new('│'))
                        .right(VerticalLine::new('│'))
                        .build(),
                )
                .separator(
                    Separator::builder()
                        .title(Some(HorizontalLine::new('├', '┤', '┼', '─')))
                        .column(Some(VerticalLine::new('│')))
                        .row(Some(HorizontalLine::new('├', '┤', '┼', '─')))
                        .build(),
                ),
        )
    }
}
