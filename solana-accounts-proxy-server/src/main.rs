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

#[cfg(test)]
mod correctness_tests {

    #[tokio::test]
    async fn get_account_info_local() {}

    /*#[tokio::test]
    async fn get_account_info_remote() {
        use std::process::Command;

        Command::new("./target/debug/solana-accounts-proxy")
            .arg("./tests/config_file/ProxyConfig.toml")
            .spawn()
            .unwrap();

        let body = json::object! {
            "jsonrpc": "2.0",
            "id": 1u8,
            "method": "getAccountInfo",
            "params": json::array![
              "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB",
              json::object!{
                "encoding": "base64",
                "commitment": "processed"
              }
            ]
        };
        let body = body.to_string();

        std::thread::sleep(std::time::Duration::from_secs(3));

        let proxy_response = minreq::post("http://127.0.0.1:4000")
            .with_header("Content-Type", "application/json")
            .with_body(body.clone())
            .send()
            .unwrap();

        let rpc_response = minreq::post("https://api.mainnet-beta.solana.com")
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .send()
            .unwrap();

        assert_eq!(
            proxy_response.as_str().unwrap(),
            rpc_response.as_str().unwrap()
        );
    }*/
}
