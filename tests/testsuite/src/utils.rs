use serde::Deserialize;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub const CONTENT_TYPE: &str = "Content-Type";
pub const APPLICATION_JSON: &str = "application/json";
pub const ARGS_ERROR: &str = "The program takes only one argument which is the path to the location of the configuration file.";

#[derive(Debug, Deserialize)]
pub struct GpaParameters {
    pubkey: String,
    parameters: Vec<Parameters>,
}

#[derive(Debug, Deserialize)]
pub struct Parameters {
    data_slice: u64,
    offset: usize,
    bytes: String,
    encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestsuiteConfig {
    rpcpool_url: String,
    data: Vec<GpaParameters>,
}

impl TestsuiteConfig {
    pub async fn load_config() -> anyhow::Result<Self> {
        let mut cli_args = std::env::args();

        if cli_args.len() > 2 {
            eprintln!("{}", ARGS_ERROR);

            std::process::exit(1);
        }

        let cli_input_path = match cli_args.nth(1) {
            None => {
                eprintln!("{}", ARGS_ERROR);

                std::process::exit(1);
            }
            Some(file_path) => file_path,
        };

        let mut file = File::open(&cli_input_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config = toml::from_str::<TestsuiteConfig>(&contents)?;

        Ok(config)
    }

    pub fn url(&self) -> &String {
        &self.rpcpool_url
    }
}

pub async fn load_binary() -> anyhow::Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    #[derive(Debug, serde::Deserialize)]
    struct Manifest {
        workspace_root: String,
    }
    let output = std::process::Command::new(env!("CARGO"))
        .arg("metadata")
        .arg("--format-version=1")
        .current_dir(manifest_dir)
        .output()
        .unwrap();

    let manifest: Manifest = serde_json::from_slice(&output.stdout).unwrap();
    let mut binary = PathBuf::new();
    binary.push(&manifest.workspace_root);

    let mut proxy_config_file = PathBuf::new();
    proxy_config_file.push(&manifest.workspace_root);
    proxy_config_file.push("tests/config_file/ProxyConfig.toml");
    dbg!(&proxy_config_file);

    #[cfg(debug_assertions)]
    binary.push("target/debug/solana-accounts-proxy-server");
    #[cfg(not(debug_assertions))]
    binary.push("target/release/solana-accounts-proxy-server");

    dbg!(&binary);

    std::process::Command::new(binary)
        .arg(proxy_config_file)
        .spawn()
        .unwrap();

    Ok(())
}
