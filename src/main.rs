#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::env;

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
async fn main() {
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

    let socket = match get_socketaddr(cli_args) {
        Ok(socket_addr) => socket_addr,
        Err(error) => {
            eprintln!("server error: {}", error); //TODO Log to facade
            std::process::exit(1);
        }
    };

    println!("Listening at socket: `{}`", socket,);

    let make_svc = make_service_fn(|_conn| async { Ok::<_, RpcProxyError>(service_fn(processor)) });

    let server = Server::bind(&socket).serve(make_svc);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e); //TODO Log to facade
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
