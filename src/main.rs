#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::Server;
use jsonrpsee::{
    core::Error as JsonRpcServerError,
    http_server::{AccessControlBuilder, HttpServerBuilder, HttpServerHandle, RpcModule},
};
use std::{collections::HashMap, env, net::SocketAddr};

#[cfg(feature = "log_with_tracing")]
use tracing_subscriber::{filter::LevelFilter, util::SubscriberInitExt};

mod requests;
pub use requests::*;

mod socket_parser;
use socket_parser::get_socketaddr;

mod rpc_traits;
pub use rpc_traits::*;

const ERROR_MESSAGE: &str =
    "Invalid Number of Command-line Arguments. Expected `1`, `2` or `4` arguments. 
Use `-h` argument for a list of commands";

const HELP_MESSAGE: [&str; 9] = [
    "solana-accounts-proxy",
    "\n",
    "   Example Usage:",
    "       solana-accounts-proxy -ip 127.0.0.1 -port 8000",
    "\n",
    "    List of arguments:",
    "       -h or --help    - Prints this help screen",
    "       -ip             - the IP address to use instead of the default IP `0.0.0.0`",
    "       -port           - the port to use instead of the default `1024`",
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli_args = env::args();

    if cli_args.len() == 2 {
        let is_help_arg = match cli_args.nth(1) {
            Some(value) => value,
            None => String::default(),
        };

        match is_help_arg.as_str() {
            "-h" | "--help" => {
                for value in HELP_MESSAGE {
                    println!("{value:10}");
                }

                std::process::exit(1);
            }
            _ => {
                eprintln!("{}", ERROR_MESSAGE);

                std::process::exit(1);
            }
        }
    }

    if cli_args.len() > 5 {
        eprintln!("{}", ERROR_MESSAGE);
        std::process::exit(1);
    }

    let socket_addr = match get_socketaddr(cli_args) {
        Ok(socket_addr) => socket_addr,
        Err(error) => {
            eprintln!("server error: {}", error); //TODO Log to facade
            std::process::exit(1);
        }
    };

    #[cfg(feature = "log_with_tracing")]
    log()?;

    let (socket_addr, server) = http_server(socket_addr).await?;
    println!("Listening at http://{:?}", socket_addr);

    server.await;

    Ok(())
}

async fn http_server(
    socket_addr: SocketAddr,
) -> Result<(SocketAddr, HttpServerHandle), jsonrpsee::core::Error> {
    let server = Server::bind(&socket_addr);

    let acl = AccessControlBuilder::new()
        .allow_all_headers()
        .allow_all_origins()
        .allow_all_hosts()
        .build();

    let server = HttpServerBuilder::new()
        .set_access_control(acl) //TODO
        .build_from_hyper(server, socket_addr)?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcProxyImpl.into_rpc())?;

    Ok((addr, handle))
}

#[cfg(feature = "log_with_tracing")]
fn log() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
        .expect("Error: The environment variable has not been passed to the crate. Try `RUST_LOG=info solana-accounts-proxy");

    tracing::info!("LOGGING WITH `tracing` crate is enabled");

    Ok(())
}
