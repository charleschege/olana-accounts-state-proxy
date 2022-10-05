use solana_accounts_proxy::{http_server, log, PgConnection, ProxyConfig, CLIENT, USER_CONFIG};

mod rng_gen;
pub use rng_gen::*;
use tokio::{fs::File, io::AsyncReadExt};

#[tokio::main]
async fn main() {
    populate_db().await;

    {
        #[cfg(all(debug_assertions, feature = "dangerous_debug",))]
        dbg!(&*USER_CONFIG);

        #[cfg(all(debug_assertions, feature = "dangerous_debug",))]
        println!(
            "POSTGRES_URL: {}",
            USER_CONFIG.postgres_config().postgres_url()
        );
    }

    log().unwrap();

    let value = PgConnection::connect(USER_CONFIG.postgres_config())
        .await
        .unwrap();

    CLIENT.write().await.replace(value);

    let (socket_addr, server) = http_server(USER_CONFIG.get_socketaddr()).await.unwrap();
    tracing::info!("Listening at http://{:?}", socket_addr);

    tokio::task::spawn(async {
        server.await;
    });
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
