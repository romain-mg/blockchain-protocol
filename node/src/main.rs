mod client_to_node_server;
mod p2pclient;
mod p2pserver;

use anyhow::Result;
use blockchain_core::{log, log::init_logger};
use clap::{Parser, Subcommand};
use p2pclient::example;
use tokio::task;

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
    Server {},
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let args = Args::parse();

    match args.command {
        Commands::Example { input } => {
            let output = example(input).await?;
            log::info!("{}", output);
            Ok(())
        }
        Commands::Server {} => {
            let p2p_handle = task::spawn(p2pserver::start());
            let client_handle = task::spawn(client_to_node_server::start());
            let _ = tokio::try_join!(p2p_handle, client_handle)?;
            Ok(())
        }
    }
}
