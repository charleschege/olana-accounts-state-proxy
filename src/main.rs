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

lazy_static! {
    static ref USER_CONFIG: ProxyConfig = load_user_config();
    static ref CLIENT: RwLock<Option<Client>> = RwLock::new(Option::None);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        #[cfg(all(debug_assertions, feature = "dangerous_debug",))]
        dbg!(&*USER_CONFIG);

        #[cfg(all(debug_assertions, feature = "dangerous_debug",))]
        println!(
            "POSTGRES_URL: {}",
            USER_CONFIG.postgres_config().postgres_url()
        );
    }

    log()?;

    match PgConnection::connect(USER_CONFIG.postgres_config()).await {
        Ok(value) => {
            CLIENT.write().await.replace(value);
        }
        Err(error) => {
            tracing::error!(
                "Unable to initialize `tokio` runtime: `{:?}`",
                error.to_string()
            );

            std::process::exit(1)
        }
    }

    let (socket_addr, server) = http_server(USER_CONFIG.get_socketaddr()).await?;
    tracing::debug!("Listening at http://{:?}", socket_addr);

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

fn log() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
        .expect("Error: The environment variable has not been passed to the crate. Try `RUST_LOG=info solana-accounts-proxy");

    tracing::info!("LOGGING WITH `tracing` crate is enabled");

    Ok(())
}
