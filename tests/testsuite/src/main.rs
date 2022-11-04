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

    let proxy_config_path = load_binary().await.unwrap();

    let config = TestsuiteConfig::load_config().await.unwrap();

    std::thread::sleep(std::time::Duration::from_secs(15));

    let para_test = ParallelTest::new(&proxy_config_path, &config);

    match para_test.run_gpa().await {
        Ok(_) => (),
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }
}
