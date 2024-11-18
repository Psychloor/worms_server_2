#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::correctness)]
#![warn(clippy::complexity)]
#![warn(clippy::suspicious)]

use crate::args::Args;
use crate::database::SHUTDOWN_TOKEN;
use clap::Parser;
use log::{error, info};
use server::Server;
use std::net::SocketAddr;

mod args;
pub(crate) mod database;
pub(crate) mod net;
pub(crate) mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> eyre::Result<()> {
    initialize_environment()?;

    handle_ctrl_c_signal();

    let args = Args::try_parse()?;
    let server_address = SocketAddr::new(args.ip, args.port);
    if let Err(e) = Server::start_server(server_address).await {
        log::error!("Server encountered an error: {}", e);
    }

    Ok(())
}

fn initialize_environment() -> eyre::Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();
    color_eyre::install()?;

    Ok(())
}

fn handle_ctrl_c_signal() {
    let cancellation_token = SHUTDOWN_TOKEN.clone();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Ctrl-C signal handler encountered an error: {}", e);
            cancellation_token.cancel();
            return;
        }
        info!("Server shutting down");
        cancellation_token.cancel();
    });
}
