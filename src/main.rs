use log::{error, info};
use server::Server;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(crate) mod database;
pub(crate) mod net;
pub(crate) mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let mut server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 17001);

    let args: Vec<_> = std::env::args().collect();
    let args_windows = args.windows(2);
    for window in args_windows {
        match window {
            [args, val] if args == "ip" => match val.parse() {
                Ok(ip) => server_address.set_ip(IpAddr::V4(ip)),
                Err(e) => {
                    error!("Error parsing ip! {}", e);
                }
            },
            [args, val] if args == "port" => match val.parse() {
                Ok(port) => server_address.set_port(port),
                Err(e) => {
                    error!("Error parsing port! {}", e);
                }
            },
            _ => {}
        }
    }

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
    if let Err(e) =
        Server::start_server(Arc::clone(&database), server_address, cancellation_token).await
    {
        error!("Error from server: {}", e);
    }

    Ok(())
}