use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

mod generate;
mod schema;
mod validate;

#[derive(Clone, ValueEnum)]
pub enum Format {
    Plain,
    Json,
}

#[derive(Parser)]
#[command(name = "envow", version, about = "Env schema validator and generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate the current environment against a schema
    Validate {
        #[arg(default_value = "envow.toml")]
        schema: PathBuf,
        /// Validate only these sections (comma-separated)
        #[arg(long, value_delimiter = ',')]
        only: Vec<String>,
        /// Environment name — skips vars scoped to other envs
        #[arg(long)]
        env: Option<String>,
        /// Output format
        #[arg(long, default_value = "plain")]
        format: Format,
    },
    /// Generate a .env.example from a schema
    Generate {
        #[arg(default_value = "envow.toml")]
        schema: PathBuf,
        #[arg(short, long, default_value = ".env.example")]
        output: PathBuf,
        /// Environment name — selects env-specific values and filters scoped vars
        #[arg(long)]
        env: Option<String>,
    },
}

fn main() {
    let exit_code = match Cli::parse().command {
        Commands::Validate { schema, only, env, format } => {
            match validate::run(&schema, &only, env.as_deref(), &format) {
                Ok(()) => 0,
                Err(validate::Error::Schema(e)) => {
                    eprintln!("error: {e}");
                    2
                }
                Err(validate::Error::Failures) => 1,
            }
        }
        Commands::Generate { schema, output, env } => {
            match generate::run(&schema, &output, env.as_deref()) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("error: {e}");
                    2
                }
            }
        }
    };

    std::process::exit(exit_code);
}
