#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::Server;
use jsonrpsee::http_server::{AccessControlBuilder, HttpServerBuilder, HttpServerHandle};
use std::{env, net::SocketAddr};

mod rpc_traits;
pub use rpc_traits::*;

mod types;
pub use types::*;

mod postgres;
pub use postgres::*;

mod config;
pub(crate) use config::*;

const ERROR_MESSAGE: &str = "Invalid Number of Command-line Arguments. Expected `2` arguments. 
Use `-h` argument for a list of commands";

const HELP_MESSAGE: [&str; 4] = [
    "solana-accounts-proxy",
    "\n",
    "   Example Usage:",
    "       solana-accounts-proxy ../configs",
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli_args = env::args();

    if cli_args.len() > 2 {
        eprintln!("{}", ERROR_MESSAGE);
        std::process::exit(1);
    }

    let cli_input_path = match cli_args.nth(1) {
        Some(path) => match path.as_str() {
            "-h" | "--help" => {
                for value in HELP_MESSAGE {
                    println!("{value:10}");
                }

                std::process::exit(1);
            }
            _ => path,
        },
        None => {
            eprintln!("Invalid commandline args. The path to the `ProxyConfig.toml` file must be passed when running the binary. Try `solana-accounts-proxy -h` for an example"); //TODO Log to facade
            std::process::exit(1);
        }
    };

    let proxy_config = match ProxyConfig::load_config(&cli_input_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("server error: {}", error); //TODO Log to facade
            std::process::exit(1);
        }
    };
    dbg!(&proxy_config);

    #[cfg(feature = "log_with_tracing")]
    log()?;

    let (socket_addr, server) = http_server(proxy_config.get_socketaddr()).await?;
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
