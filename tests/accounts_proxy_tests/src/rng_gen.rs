use nanorand::{BufferedRng, Rng, WyRand};
use std::fmt;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn populate_db() {
    let mut owners = Vec::with_capacity(10);
    (0..10).for_each(|_| {
        owners.push(base58_rand_string());
    });

    let mut file = File::open("./pg_env.txt").await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();

    let pool = sqlx::PgPool::connect(&contents).await.unwrap();

    for _ in 0..100 {
        let block = Block::rand(owners.as_ref());

        sqlx::query(
            "INSERT INTO slot(slot, parent, status)
                VALUES($1, $2, $3)
            ",
        )
        .bind(block.slot)
        .bind(0i64)
        .bind(block.status)
        .execute(&pool)
        .await
        .unwrap();

        for account in block.accounts {
            sqlx::query(
                "
            INSERT INTO account_write(pubkey, slot, write_version, owner, is_selected, lamports, executable, rent_epoch, data)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ",
            )
            .bind(account.pubkey)
            .bind(block.slot)
            .bind(account.write_version)
            .bind(account.owner)
            .bind(account.is_selected)
            .bind(account.lamports)
            .bind(account.executable)
            .bind(account.rent_epoch)
            .bind(account.data)
            .execute(&pool)
            .await
            .unwrap();
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
pub enum SlotStatus {
    Processed,
    Confirmed,
    Rooted,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    status: SlotStatus,
    accounts: Vec<TempAccountInfo>,
    slot: i64,
}

impl Block {
    pub fn rand(owners: &[String]) -> Self {
        let mut accounts = Vec::<TempAccountInfo>::new();

        (0..10).for_each(|_| {
            accounts.push(TempAccountInfo::rand(owners));
        });

        Block {
            status: rand_status(),
            accounts,
            slot: rand_i64(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TempAccountInfo {
    pubkey: String,
    write_version: i64,
    owner: String,
    is_selected: bool,
    lamports: i64,
    executable: bool,
    rent_epoch: i64,
    data: Vec<u8>,
}

impl fmt::Debug for TempAccountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TempAccountInfo")
            .field("pubkey", &self.pubkey)
            .field("write_version", &self.write_version)
            .field("owner", &self.owner)
            .field("is_selected", &self.is_selected)
            .field("lamports", &self.lamports)
            .field("executable", &self.executable)
            .field("rent_epoch", &self.rent_epoch)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}

impl TempAccountInfo {
    pub fn rand(owners: &[String]) -> Self {
        Self {
            pubkey: base58_rand_string(),
            owner: select_owner(owners),
            write_version: rand_i64(),
            is_selected: true,
            lamports: rand_i64(),
            executable: rand_bool(),
            rent_epoch: rand_i64(),
            data: rand_bytes::<30>().to_vec(),
        }
    }
}

pub fn select_owner(owners: &[String]) -> String {
    let mut rng = WyRand::new();

    let num = rng.generate_range(0..=9usize);
    owners[num].clone()
}

pub fn rand_bytes<const T: usize>() -> [u8; T] {
    let mut buffer = [0u8; T];
    let mut rng = BufferedRng::new(WyRand::new());

    rng.fill(&mut buffer);

    buffer
}

pub fn base58_rand_string() -> String {
    bs58::encode(&rand_bytes::<32>()).into_string()
}

pub fn rand_i64() -> i64 {
    let mut rng = WyRand::new();
    let random = rng.generate::<u64>() as i64;
    random.abs()
}

pub fn rand_bool() -> bool {
    let mut rng = WyRand::new();

    let num = rng.generate_range(0..=1u8);

    !num == 0
}

pub fn rand_status() -> SlotStatus {
    let mut rng = WyRand::new();

    let num = rng.generate_range(0..=2u8);

    match num {
        1 => SlotStatus::Confirmed,
        2 => SlotStatus::Rooted,
        _ => SlotStatus::Processed,
    }
}
