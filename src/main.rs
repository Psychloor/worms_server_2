use crate::args::Args;
use crate::database::SHUTDOWN_TOKEN;
use clap::Parser;
use server::Server;
use std::net::SocketAddr;

mod args;
pub(crate) mod database;
pub(crate) mod net;
pub(crate) mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    initialize_environment();

    handle_ctrl_c_signal();

    let args = Args::try_parse()?;
    let server_address = SocketAddr::new(args.ip, args.port);
    if let Err(e) = Server::start_server(server_address).await {
        log::error!("Server encountered an error: {}", e);
    }

    Ok(())
}

fn initialize_environment() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
}

fn handle_ctrl_c_signal() {
    let cancellation_token = SHUTDOWN_TOKEN.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C signal!");
        log::info!("Server shutting down");
        cancellation_token.cancel();
    });
}