use sea_orm::{Database, DatabaseBackend, MockDatabase};
/* 
mod account_info;
pub use account_info::{Entity as AccountInfoEntity, Model as AccountInfoModel};

/// A test database
pub async fn check_mock_connection() {
    //let dbconn = Database::connect("postgres://root:root@localhost:5432").await?;

    let mock_dbconn = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![AccountInfoModel {
            account_id: 1,
            key: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_owned(),
            is_signer: true,
            is_writable: true,
            lamports: 4_000_000_000,
            data: Vec::default(),
            owner: "11111111111111111111111111111111".to_owned(),
            executable: false,
            rent_epoch: 50,
        }]])
        .into_connection();
}

pub type Base58PublicKey = String;

*/
