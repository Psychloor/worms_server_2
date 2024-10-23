use crate::args::Args;
use anyhow::anyhow;
use clap::Parser;
use log::{error, info};
use server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

mod args;
pub(crate) mod database;
pub(crate) mod net;
pub(crate) mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let args = Args::try_parse().map_err(|e| anyhow!("Error parsings args!").context(e))?;

    let cancellation_token = CancellationToken::new();
    let ct_clone = cancellation_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl + C!");
        info!("Server shutting down");
        ct_clone.cancel();
    });

    let database = database::Database::new();
    let server_address = SocketAddr::new(args.ip, args.port);
    if let Err(e) =
        Server::start_server(Arc::clone(&database), server_address, cancellation_token).await
    {
        error!("Error from server: {}", e);
    }

    Ok(())
}
