#![forbid(unsafe_code)]

use std::path::Path;

mod gpa_tests;
pub use gpa_tests::*;

mod ga;
pub use ga::*;

mod utils;
pub use utils::*;

#[tokio::main]
async fn main() {
    let proxy_config_path = load_binary().await.unwrap();

    let config = TestsuiteConfig::load_config().await.unwrap();

    //test_ga(&proxy_config_path, &config).await;

    test_gpa(&proxy_config_path, &config).await;
}

async fn test_ga(proxy_config_path: &Path, config: &TestsuiteConfig) {
    let pubkey = "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD";
    let encoding = "base64";
    let commitment = "finalized";

    let ga_data = Ga::new()
        .add_pubkey(pubkey)
        .add_commitment(commitment)
        .add_encoding(encoding);
    let rpcpool_outcome = ga_data.req_from_rpcpool(config).await.unwrap();

    dbg!(&rpcpool_outcome);

    let proxy_outcome = ga_data.req_from_proxy(proxy_config_path).await.unwrap();

    dbg!(&proxy_outcome);
}

async fn test_gpa(proxy_config_path: &Path, config: &TestsuiteConfig) {
    let program_id = "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD";
    let offset_public_key = "CyZuD7RPDcrqCGbNvLCyqk6Py9cEZTKmNKujfPi3ynDd";
    let offset = 32;
    let data_size = 165;
    let encoding = "base64";
    let commitment = "finalized";

    let mut gpa_tests = GetProgramAccountsTests::new();
    gpa_tests
        .add_program_id(program_id)
        //.add_offset_public_key(offset_public_key)
        //.add_data_size(data_size)
        //.add_offset(offset)
        .add_commitment(commitment)
        .add_encoding(encoding)
        .add_with_context(true);

    let rpcpool_outcome = gpa_tests.req_from_rpcpool(config).await.unwrap();

    let proxy_outcome = gpa_tests.req_from_proxy(proxy_config_path).await.unwrap();

    println!(
        "PROXY SLOT [{}] - RPCPOOL SLOT [{}]",
        proxy_outcome.result.context.slot, rpcpool_outcome.result.context.slot
    );
    println!(
        "PROXY RESULT VEC LEN   - {:?}",
        proxy_outcome.result.value.len()
    );
    println!(
        "RPCPOOL RESULT VEC LEN - {:?}",
        rpcpool_outcome.result.value.len()
    );

    assert_eq!(rpcpool_outcome.jsonrpc, proxy_outcome.jsonrpc,);
    assert_eq!(rpcpool_outcome.id, proxy_outcome.id,);

    let mut matched_counter = 0usize;

    let mut unmatched = Vec::<String>::new();
    for value in &proxy_outcome.result.value {
        let current_result = rpcpool_outcome
            .result
            .value
            .iter()
            .enumerate()
            .find(|(_index, account)| account.account.data == value.account.data);

        match current_result {
            Some(_) => {
                matched_counter += 1;
            }
            None => {
                unmatched.push(value.pubkey.clone());
            }
        }
    }

    println!("MATCHED   PUBLIC KEYS: {}", matched_counter);
    println!("UNMATCHED PUBLIC KEYS: {}", unmatched.len());

    println!("PUBLIC KEYS MISMATCHED DATA FROM PROXY SERVER");
    dbg!(&unmatched);

    for target in &unmatched {
        let pool_acc = rpcpool_outcome
            .result
            .value
            .iter()
            .find(|account| &account.pubkey == target);

        let proxy_acc = proxy_outcome
            .result
            .value
            .iter()
            .find(|account| &account.pubkey == target);

        assert_eq!(pool_acc.unwrap().pubkey, proxy_acc.unwrap().pubkey);
        assert_eq!(
            pool_acc.unwrap().account.executable,
            proxy_acc.unwrap().account.executable
        );
        assert_eq!(
            pool_acc.unwrap().account.owner,
            proxy_acc.unwrap().account.owner
        );
        assert_eq!(
            pool_acc.unwrap().account.lamports,
            proxy_acc.unwrap().account.lamports
        );
        assert_eq!(
            pool_acc.unwrap().account.rent_epoch,
            proxy_acc.unwrap().account.rent_epoch
        );
        assert_eq!(
            blake3::hash(pool_acc.unwrap().account.data.0.as_bytes()),
            blake3::hash(proxy_acc.unwrap().account.data.0.as_bytes())
        );
    }

    assert!(unmatched.is_empty());
}
