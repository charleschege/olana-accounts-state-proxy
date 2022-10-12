#![forbid(unsafe_code)]

use std::path::PathBuf;

mod gpa_tests;
pub use gpa_tests::*;

mod utils;
pub use utils::*;

#[tokio::main]
async fn main() {
    load_binary().await.unwrap();

    let program_id = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    let offset_public_key = "CyZuD7RPDcrqCGbNvLCyqk6Py9cEZTKmNKujfPi3ynDd";
    let offset = 32;
    let data_size = 165;
    let encoding = "base64";

    let mut gpa_tests = GetProgramAccountsTests::new();
    gpa_tests
        .add_program_id(program_id)
        .add_offset_public_key(offset_public_key)
        .add_data_size(data_size)
        .add_offset(offset)
        .add_encoding(encoding);

    dbg!(&gpa_tests.to_json_string());
    dbg!(&gpa_tests.to_json_string().len());

    let config = load_config().await;

    gpa_tests.req_from_rpcpool(&config).await.unwrap();
}
