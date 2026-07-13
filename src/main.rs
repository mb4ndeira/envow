use std::path::PathBuf;
use clap::{Parser, Subcommand};

mod generate;
mod schema;
mod validate;

#[derive(Parser)]
#[command(name = "envolve", version, about = "Env schema validator and generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate the current environment against a schema
    Validate {
        #[arg(default_value = "envolve.toml")]
        schema: PathBuf,
        /// Only validate these sections (comma-separated)
        #[arg(long, value_delimiter = ',')]
        only: Vec<String>,
    },
    /// Generate a .env.example from a schema
    Generate {
        #[arg(default_value = "envolve.toml")]
        schema: PathBuf,
        #[arg(short, long, default_value = ".env.example")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Validate { schema, only } => validate::run(&schema, &only),
        Commands::Generate { schema, output } => generate::run(&schema, &output),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
