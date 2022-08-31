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

    http_server(socket_addr).await?.await;

    Ok(())
}

async fn http_server(socket_addr: SocketAddr) -> Result<HttpServerHandle, jsonrpsee::core::Error> {
    let server = Server::bind(&socket_addr);

    println!("Listening at http://{:?}", socket_addr);

    let acl = AccessControlBuilder::new()
        .allow_all_headers()
        .allow_all_origins()
        .allow_all_hosts()
        .build();

    let mut module = RpcModule::new(());
    module.register_method("getAccountInfo", |params, _| {
        let parameters = params.parse::<(String, HashMap<String, String>)>()?;

        let _public_key = match bs58::decode(&parameters.0).into_vec() {
            Ok(public_key) => public_key,
            Err(error) => {
                let mut base58_error = String::new();
                base58_error.push_str("Invalid Base58 Public Key. Error: `");
                base58_error.push_str(error.to_string().as_str());
                base58_error.push_str("`.");

                #[cfg(feature = "log_with_tracing")]
                tracing::info!("{}", &base58_error);

                return Err::<String, JsonRpcServerError>(JsonRpcServerError::Custom(base58_error));
            }
        };

        let _parse_parameters = match Parameter::parse(&parameters.1) {
            Ok(values) => values,
            Err(error) => {
                #[cfg(feature = "log_with_tracing")]
                tracing::info!("{}", &error);

                return Err::<String, JsonRpcServerError>(JsonRpcServerError::Custom(error));
            }
        };

        if let Some(encoding_value) = parameters.1.get("encoding") {
            let encoding: Encoding = encoding_value.as_str().into();

            match encoding.is_supported() {
                Ok(_) => (),
                Err(error) => {
                    #[cfg(feature = "log_with_tracing")]
                    tracing::info!("{}", &error);

                    return Err::<String, JsonRpcServerError>(JsonRpcServerError::Custom(error));
                }
            }
        }

        let data_from_db = "DATA PROCESSED....";
        #[cfg(feature = "log_with_tracing")]
        tracing::info!("{}", data_from_db);

        Ok(data_from_db.to_owned())
    })?;

    HttpServerBuilder::new()
        .set_access_control(acl) //TODO
        .build_from_hyper(server, socket_addr)?
        .start(module)
}

#[cfg(feature = "log_with_tracing")]
fn log() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?
        .add_directive(LevelFilter::INFO.into());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    tracing::info!("LOGGING WITH `tracing` crate is enabled");

    Ok(())
}
