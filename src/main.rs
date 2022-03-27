use anyhow::Result;
use bb::AppBuilder;
use builders::{Builder, NpmBuilder, YarnBuilder};
use clap::{Parser, Subcommand};
mod bb;
mod builders;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build an app directory into an image
    Build {
        /// Directory of app source
        #[clap(required = true)]
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { path } => build(path),
    }
}

pub fn build(path: &String) -> Result<()> {
    let builders: Vec<Box<dyn Builder>> = vec![Box::new(YarnBuilder {}), Box::new(NpmBuilder {})];

    let mut app_builder = AppBuilder::new(path.to_string());
    app_builder.detect(&builders)?;

    app_builder.build()?;

    Ok(())
}
