#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use jsonrpsee::{
    core::rpc_params,
    http_server::{AccessControlBuilder, HttpServerBuilder, HttpServerHandle, RpcModule},
};
use socket2::{Domain, Socket, Type};
use std::{
    env,
    net::{SocketAddr, TcpListener},
};
use tracing_subscriber::{filter::LevelFilter, util::SubscriberInitExt};

mod requests;
pub use requests::*;

mod errors;
pub use errors::*;

mod socket_parser;
use socket_parser::get_socketaddr;

mod http_parser;
pub use http_parser::*;

const ERROR_MESSAGE: &str = "Invalid Number of Command-line Arguments. 
Expected `1`, `2` or `4` arguments. 
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
                println!("{}", ERROR_MESSAGE);

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

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?
        .add_directive(LevelFilter::INFO.into());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    dbg!(http_server(socket_addr).await?.await);

    Ok(())
}

async fn http_server(socket_addr: SocketAddr) -> Result<HttpServerHandle, jsonrpsee::core::Error> {
    let server = Server::bind(&socket_addr);

    tracing::info!("{:?}", socket_addr);

    let acl = AccessControlBuilder::new()
        .allow_all_headers()
        .allow_all_origins()
        .allow_all_hosts()
        .build();

    let mut module = RpcModule::new(());
    module.register_method("getAccountInfo", |_, _| {
        tracing::info!("AccountInfo method invoked");
        Ok("Processed RPC method")
    })?;

    HttpServerBuilder::new()
        .set_access_control(acl) //TODO
        .set_middleware(RpcProxyMiddleware)
        .build_from_hyper(server, socket_addr)?
        .start(module)
}
