#![forbid(unsafe_code)]

mod gpa_tests;
pub use gpa_tests::*;

mod utils;
pub use utils::*;

#[tokio::main]
async fn main() {
    let proxy_config_path = load_binary().await.unwrap();

    let program_id = "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD";
    let offset_public_key = "CyZuD7RPDcrqCGbNvLCyqk6Py9cEZTKmNKujfPi3ynDd";
    let offset = 32;
    let data_size = 165;
    let encoding = "base64";
    let commitment = "confirmed";

    let mut gpa_tests = GetProgramAccountsTests::new();
    gpa_tests
        .add_program_id(program_id)
        //.add_offset_public_key(offset_public_key)
        //.add_data_size(data_size)
        //.add_offset(offset)
        .add_commitment(commitment)
        .add_encoding(encoding);

    let config = TestsuiteConfig::load_config().await.unwrap();

    let rpcpool_outcome = gpa_tests.req_from_rpcpool(&config).await.unwrap();

    let proxy_outcome = gpa_tests.req_from_proxy(&proxy_config_path).await.unwrap();

    println!("PROXY RESULT LEN   - {:?}", proxy_outcome.result.len());
    println!("RPCPOOL RESULT LEN - {:?}", rpcpool_outcome.result.len());

    assert_eq!(rpcpool_outcome.jsonrpc, proxy_outcome.jsonrpc,);
    assert_eq!(rpcpool_outcome.id, proxy_outcome.id,);

    let mut unmatched = Vec::<String>::new();
    for value in proxy_outcome.result {
        let current_result = rpcpool_outcome
            .result
            .iter()
            .enumerate()
            .find(|(_index, account)| account == &&value);

        match current_result {
            Some(_) => (),
            None => {
                unmatched.push(value.pubkey);
            }
        }
    }

    println!("UNMATCHED PUBLIC KEYS FROM PROXY SERVER");
    dbg!(&unmatched);

    assert!(unmatched.is_empty());
}
