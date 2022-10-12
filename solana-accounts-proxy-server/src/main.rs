use solana_accounts_proxy::{http_server, log, PgConnection, CLIENT, USER_CONFIG};

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
    tracing::info!("Listening at http://{:?}", socket_addr);

    server.await;

    Ok(())
}
