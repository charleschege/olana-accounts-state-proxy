use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncReadExt};

pub const CONTENT_TYPE: &str = "Content-Type";
pub const APPLICATION_JSON: &str = "application/json";
pub const ARGS_ERROR: &str = "The program takes only one argument which is the path to the location of the configuration file.";

#[derive(Debug, Deserialize, Clone)]
pub struct TestsuiteConfig {
    pub rpcpool_url: String,
    pub binary_name: String,
    pub proxy_config_file: PathBuf,
    pub ga_data: GaData,
    /// Vec<(ProgramID, GpaData)>
    pub gpa_data: Vec<(String, GpaData)>,
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

        let config = json5::from_str::<TestsuiteConfig>(&contents)?;

        Ok(config)
    }

    pub fn url(&self) -> &String {
        &self.rpcpool_url
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GaData {
    pub pubkey: String,
    pub commitment: String,
    pub encoding: String,
    pub data_slice: DataSlice,
    pub min_context_slot: usize,
}

impl GaData {
    pub fn new() -> Self {
        GaData::default()
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GpaData {
    pub offset_public_key: String,
    pub commitment: String,
    pub encoding: String,
    pub data_slice: DataSlice,
    pub min_context_slot: usize,
    pub bytes: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSlice {
    pub offset: usize,
    pub length: usize,
}

/// Return a [Result] containing the path to the configuration
/// file of the proxy server
pub async fn load_binary(proxy_config: &Path, binary_name: &Path) -> anyhow::Result<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    #[derive(Debug, serde::Deserialize)]
    struct Manifest {
        workspace_root: String,
    }
    let output = std::process::Command::new(env!("CARGO"))
        .arg("metadata")
        .arg("--format-version=1")
        .current_dir(manifest_dir)
        .output()?;

    let manifest: Manifest = serde_json::from_slice(&output.stdout)?;
    let mut binary = PathBuf::new();
    binary.push(&manifest.workspace_root);

    let mut proxy_config_file = PathBuf::new();
    proxy_config_file.push(&manifest.workspace_root);
    proxy_config_file.push(proxy_config);

    #[cfg(debug_assertions)]
    binary.push(path_builder("debug", binary_name));
    #[cfg(not(debug_assertions))]
    binary.push(path_builder("release", binary_name));

    tracing::info!("LOADED BINARY PATH: {:?}", &binary);
    tracing::info!("LOADED PROXY CONFIG FILE: {:?}", &proxy_config_file);

    std::process::Command::new(binary)
        .arg(proxy_config_file.clone())
        .spawn()?;

    Ok(proxy_config_file)
}

fn path_builder(compile_mode: &str, binary_name: &Path) -> PathBuf {
    let mut relative_path = PathBuf::new();

    relative_path.push("target");
    relative_path.push(compile_mode);
    relative_path.push(binary_name);

    relative_path
}
