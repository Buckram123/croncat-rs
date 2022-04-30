use std::{process::exit};

use croncat::{logging::{self, info}, errors::Report, tokio, grpc, ws};

mod cli;
mod opts;
mod env;

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Get environment variables
    let env = env::load()?;

    // Setup tracing and error reporting
    logging::setup()?;
    
    // Get the CLI options, handle argument errors nicely
    let opts = cli::get_opts().map_err(|e| {
        println!("{}", e);
        exit(1);
    }).unwrap();

    // If there ain't no no-frills...
    if !opts.no_frills {
        cli::print_banner();
    }

    info!("Starting croncatd...");

    // Create a channel to handle graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = cli::create_shutdown_channel();
    
    // Connect to GRPC
    let (_msg_client, _query_client) = grpc::connect(env.grpc_url.clone()).await?;

    // Stream new blocks from the WS RPC subscription
    let block_stream = tokio::task::spawn(async move {
        ws::stream_blocks(env.wsrpc_url.clone(), &mut shutdown_rx).await.expect("Failed");
    });

    // Handle SIGINT AKA Ctrl-C
    let ctrl_c = tokio::task::spawn( async move {
        tokio::signal::ctrl_c().await.expect("Failed to wait for Ctrl-C");
        shutdown_tx.send(()).await.expect("Failed to send shutdown signal");
        println!("");
        info!("Shutting down croncatd...");
    });

    // TODO: Do something with the main_loop return value
    let (_, _) = tokio::join!(ctrl_c, block_stream);

    // Say goodbye if no no-banner
    if !opts.no_frills {
        println!("\n🐱 Cron Cat says: Goodbye / さようなら\n");
    }

    Ok(())
}

