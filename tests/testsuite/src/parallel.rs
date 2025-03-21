use std::path::PathBuf;

use crate::{Ga, RpcAccount, RpcAccountInfo, TestsuiteConfig};
use solana_accounts_proxy::{RpcResult, WithContext};

type GaResponse = RpcResult<WithContext<RpcAccount>>;

type GpaResponse = RpcResult<WithContext<Vec<RpcAccountInfo>>>;

#[derive(Debug)]
pub struct ParallelTest<'gpa> {
    testsuite_config: &'gpa TestsuiteConfig,
    proxy_file_absolute_path: PathBuf,
}

impl<'gpa> ParallelTest<'gpa> {
    pub fn new(testsuite_config: &'gpa TestsuiteConfig, proxy_file_absolute_path: PathBuf) -> Self {
        ParallelTest {
            testsuite_config,
            proxy_file_absolute_path,
        }
    }

    pub async fn run_ga(self) -> anyhow::Result<()> {
        let pubkey = "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD";
        let encoding = "base64";
        let commitment = "finalized";

        let ga_data = Ga::new()
            .add_pubkey(pubkey)
            .add_commitment(commitment)
            .add_encoding(encoding);

        let spawn_ga_data = ga_data.clone();
        let spawn_config = self.testsuite_config.clone();

        let rpcpool_thread = tokio::spawn(async move {
            tracing::debug!("Fetching `getAccountInfo` for rpcpool in thread");
            let rpcpool_outcome = spawn_ga_data.req_from_rpcpool(spawn_config).await?;
            tracing::debug!("Finished running thread rpcpool in thread");

            Ok::<GaResponse, anyhow::Error>(rpcpool_outcome)
        });

        dbg!(&self);

        let proxy_outcome = ga_data
            .req_from_proxy(&self.testsuite_config.proxy_config_file)
            .await?;

        let rpcpool_outcome = rpcpool_thread.await??;

        println!(
            "PROXY SLOT [{}] - RPCPOOL SLOT [{}]",
            proxy_outcome.result.context.slot, rpcpool_outcome.result.context.slot
        );

        Ok(())
    }

    pub async fn run_gpa(&self) -> anyhow::Result<()> {
        //TODO Iterate over all values
        let gpa_data = self.testsuite_config.gpa_data[0].clone();

        let mut gpa_tests = crate::GetProgramAccountsTests::new();
        gpa_tests
            .add_program_id(&gpa_data.0.clone())
            //.add_offset_public_key(offset_public_key)
            //.add_data_size(data_size)
            //.add_offset(offset)
            .add_commitment(&gpa_data.1.commitment)
            .add_encoding(&gpa_data.1.encoding)
            .add_with_context(true);

        let spawn_gpa_data = gpa_tests.clone();
        let spawn_config = self.testsuite_config.clone();

        let rpcpool_thread = tokio::spawn(async move {
            tracing::debug!("Fetching `getProgramAccounts` for rpcpool in thread");
            let rpcpool_outcome = spawn_gpa_data.req_from_rpcpool(&spawn_config).await?;
            tracing::debug!("Finished running a thread `gPA`");

            Ok::<GpaResponse, anyhow::Error>(rpcpool_outcome)
        });

        let proxy_outcome = gpa_tests
            .req_from_proxy(&self.proxy_file_absolute_path)
            .await?;

        let rpcpool_outcome = rpcpool_thread.await??;

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

        Ok(())
    }
}
