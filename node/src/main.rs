mod client_to_node_server;
mod p2pclient;
mod p2pserver;

use anyhow::Result;
use blockchain_core::{
    log,
    log::{error, init_logger},
};
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
            let client_handle = task::spawn(async {
                if let Err(e) = client_to_node_server::start().await {
                    error!("client_to_node_server failed: {:?}", e);
                    if let Err(e) = client_to_node_server::start_secondary().await {
                        error!("client_to_node_server_secondary failed: {:?}", e);
                        Err(e)
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            });
            let p2p_handle = task::spawn(async {
                if let Err(e) = p2pserver::start().await {
                    error!("p2pserver failed: {:?}", e);
                    if let Err(e) = p2pserver::start_secondary().await {
                        error!("p2pserver_secondary failed: {:?}", e);
                        Err(e)
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            });

            let _ = tokio::try_join!(p2p_handle, client_handle)?;
            Ok(())
        }
    }
}
