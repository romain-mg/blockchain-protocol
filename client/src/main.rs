pub mod client;

use anyhow::Result;
use blockchain_core::{log, log::init_logger};
use clap::{Parser, Subcommand};
use client::example;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Example {
        #[clap(value_parser)]
        input: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let args = Args::parse();

    match args.command {
        Commands::Example { input } => {
            let output = example(input).await?;
            log::info!("{}", output);
        }
    }

    Ok(())
}
