#![forbid(unsafe_code)]

use solana_accounts_proxy::log;

mod gpa_tests;
pub use gpa_tests::*;

mod ga;
pub use ga::*;

mod utils;
pub use utils::*;

mod parallel;
pub use parallel::*;

#[tokio::main]
async fn main() {
    log().unwrap();

    let config = TestsuiteConfig::load_config().await.unwrap();

    let proxy_file_absolute_path = load_binary(
        &config.proxy_config_file,
        std::path::Path::new(&config.binary_name),
    )
    .await
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(15));

    let para_test = ParallelTest::new(&config, proxy_file_absolute_path);

    match para_test.run_gpa().await {
        Ok(_) => (),
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }
}
