use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Specific ip address to listen on
    #[arg(short, long, default_value = "0.0.0.0")]
    pub(crate) ip: IpAddr,

    /// Specific port to listen to, otherwise any assigned one
    #[arg(short, long, default_value = "17000")]
    pub(crate) port: u16,
}
