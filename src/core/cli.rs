use super::{
    Config, Schematic, add_schematic, generate_schematic, remove_schematic, show_schematics,
};

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, default_value_t = false, help = "Enable verbose output")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Args)]
pub struct LocalArgs {
    #[arg(help = "Name of the schematic")]
    name: String,

    #[arg(short, long, help = "Absolute path of the schematic directory")]
    path: String,

    #[arg(short, long, help = "Description of the schematic")]
    description: Option<String>,
}

#[derive(Debug, Args)]
pub struct RemoteArgs {
    #[arg(help = "Name of the schematic")]
    name: String,

    #[arg(short, long, help = "Git repository URL of the schematic")]
    url: String,

    #[arg(short, long, help = "Git branch of the schematic repository")]
    branch: Option<String>,

    #[arg(short, long, help = "Description of the schematic")]
    description: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum AddSchematicCommand {
    Local(LocalArgs),
    Remote(RemoteArgs),
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "List all available schematics", long_about = None)]
    List,
    #[command(subcommand, alias = "add")]
    AddSchematic(AddSchematicCommand),
    #[command(about = "List all available schematics", long_about = None, alias = "remove")]
    RemoveSchematic {
        #[arg(help = "Name of the schematic to remove")]
        name: String,
    },
    #[command(about = "List all available schematics", long_about = None, alias = "g")]
    Generate {
        #[arg(help = "Name of the schematic to generate")]
        name: String,
        #[arg(
            short = 'o',
            long,
            help = "Destination path for the generated schematic"
        )]
        destination: Option<String>,
    },
}

pub fn cli(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    match &args.command {
        Commands::List => {
            return show_schematics(&config.schematics);
        }
        Commands::AddSchematic(c) => match c {
            AddSchematicCommand::Local(args) => {
                return add_schematic(
                    config,
                    args.name.clone(),
                    Schematic::new_local(args.path.clone(), args.description.clone()),
                );
            }
            AddSchematicCommand::Remote(args) => {
                return add_schematic(
                    config,
                    args.name.clone(),
                    Schematic::new_remote(
                        args.url.clone(),
                        args.branch.clone(),
                        args.description.clone(),
                    ),
                );
            }
        },
        Commands::RemoveSchematic { name } => {
            return remove_schematic(config, name);
        }
        Commands::Generate { name, destination } => {
            return generate_schematic(&config.schematics, name, destination);
        }
    }
}
