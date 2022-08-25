#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use core::task::{Context, Poll};
use futures::ready;
use hyper::{
    server::{
        accept::Accept,
        conn::{AddrIncoming, AddrStream},
    },
    service::{make_service_fn, service_fn},
    Server,
};
use std::{
    env, fs,
    future::Future,
    io::{self, ErrorKind},
    path::PathBuf,
    pin::Pin,
    sync,
    sync::Arc,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::ServerConfig;

mod requests;
pub use requests::*;

mod errors;
pub use errors::*;

mod http_parser;
pub use http_parser::*;

mod config_loader;
use config_loader::*;

fn main() {
    if let Err(e) = run_server() {
        eprintln!("ENCOUNTERED AN ERROR: {}", e);
        std::process::exit(1);
    }
}

#[tokio::main]
async fn run_server() -> RpcProxyResult<()> {
    let config_file = match env::args().nth(1) {
        None => return Err(RpcProxyError::MissingPathToConfigFile),
        Some(path) => path,
    };
    let config = match Config::load_file(&config_file) {
        Ok(value) => value,
        Err(error) => match error {
            RpcProxyError::Io(io_error) => {
                if io_error == ErrorKind::NotFound {
                    eprintln!("Config file `ProxyConfig.toml` not found in provided path");

                    std::process::exit(1);
                } else {
                    return Err(error);
                }
            }
            _ => return Err(error),
        },
    };

    let socket = config.get_socketaddr()?;
    let public_key = config.get_public();
    let private_key = config.get_private();

    let tls_cfg = {
        let certs = load_certs(&public_key)?;
        let key = load_private_key(&private_key)?;
        let mut cfg = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        sync::Arc::new(cfg)
    };

    let incoming = AddrIncoming::bind(&socket)?;
    let service = make_service_fn(|_| async { Ok::<_, RpcProxyError>(service_fn(processor)) });
    let server = Server::builder(TlsAcceptor::new(tls_cfg, incoming)).serve(service);

    println!("Listening on https://{}.", socket);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e); //TODO Log to facade
    }

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

enum State {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

struct TlsStream {
    state: State,
}

impl TlsStream {
    fn new(stream: AddrStream, config: Arc<ServerConfig>) -> TlsStream {
        let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
        TlsStream {
            state: State::Handshaking(accept),
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

struct TlsAcceptor {
    config: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl TlsAcceptor {
    fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> TlsAcceptor {
        TlsAcceptor { config, incoming }
    }
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

fn load_certs(filename: &PathBuf) -> RpcProxyResult<Vec<rustls::Certificate>> {
    let certfile = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut reader)?;

    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

fn load_private_key(filename: &PathBuf) -> RpcProxyResult<rustls::PrivateKey> {
    let keyfile = fs::File::open(filename)?;
    let mut reader = io::BufReader::new(keyfile);

    let keys = rustls_pemfile::rsa_private_keys(&mut reader)?;
    if keys.len() != 1 {
        return Err(RpcProxyError::Custom(
            "expected a single private key".into(),
        ));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}
