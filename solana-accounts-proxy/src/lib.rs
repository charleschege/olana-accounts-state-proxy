#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::Server;
use jsonrpsee::http_server::{AccessControlBuilder, HttpServerBuilder, HttpServerHandle};
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::sync::RwLock;
use tokio_postgres::Client;

mod rpc;
pub use rpc::*;

mod types;
pub use types::*;

mod postgres;
pub use postgres::*;

mod config;
pub use config::*;

mod errors;
pub use errors::*;

lazy_static! {
    /// Reads the user configuration and stores it in a global static variable
    pub static ref USER_CONFIG: ProxyConfig = load_user_config();
    /// parses a user supplied file and stores it in a global static variable.
    /// Useful for testing purposes
    /// Stores a global static ref to the postgres database `Client`
    pub static ref CLIENT: RwLock<Option<Client>> = RwLock::new(Option::None);
}

/// Create a HTTP server to serve RPC requests
pub async fn http_server(
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
        .max_response_body_size(u32::MAX)
        .build_from_hyper(server, socket_addr)?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcProxyImpl.into_rpc())?;

    Ok((addr, handle))
}

/// Enable the logger
pub fn log() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
        .expect("Error: The environment variable has not been passed to the crate. Try `RUST_LOG=info solana-accounts-proxy");

    tracing::info!("LOGGING WITH `tracing` crate is enabled");

    Ok(())
}
