use crate::{Commitment, Context, GetAccountInfoRow, ProxyResult};

/// Helper struct to create the query for `getAccountInfo` using the builder pattern
pub struct GetAccountInfoQuery<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
    min_context_slot: Option<u64>,
}

impl<'q> GetAccountInfoQuery<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetAccountInfoQuery {
            base58_public_key: "",
            commitment: "",
            min_context_slot: Option::None,
        }
    }

    /// Add a base58 public key
    pub fn add_public_key(&mut self, base58_public_key: &'q str) -> &mut Self {
        self.base58_public_key = base58_public_key;

        self
    }

    /// Add the commitment level
    pub fn add_commitment(&mut self, commitment: &'q str) -> &mut Self {
        self.commitment = commitment;

        self
    }

    /// Add the minimum context slot
    pub fn add_min_context_slot(&mut self, min_context_slot: Option<u64>) -> &mut Self {
        self.min_context_slot = min_context_slot;

        self
    }

    /// Build the SQL query
    pub async fn query(self) -> ProxyResult<GetAccountInfoRow> {
        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let pubkey = self.base58_public_key;

        if let Some(min_context_slot) = self.min_context_slot {
            let slot = min_context_slot as i64;

            let row = pg_client
                .query_one(
                    "
                    SELECT 
                        account_write.slot,
                        account_write.data,
                        account_write.executable,
                        account_write.owner,
                        account_write.lamports,
                        account_write.rent_epoch
                    FROM account_write WHERE pubkey = '$1'
                    AND slot >= $2;",
                    &[&pubkey, &slot],
                )
                .await?;

            let outcome: GetAccountInfoRow = row.into();

            Ok(outcome)
        } else {
            let row = pg_client
                .query_one(
                    "
                    SELECT 
                        account_write.slot,
                        account_write.data,
                        account_write.executable,
                        account_write.owner,
                        account_write.lamports,
                        account_write.rent_epoch
                    FROM account_write WHERE pubkey = $1::TEXT;",
                    &[&pubkey],
                )
                .await?;

            let outcome: GetAccountInfoRow = row.into();

            Ok(outcome)
        }
    }
}

impl<'q> Default for GetAccountInfoQuery<'q> {
    fn default() -> Self {
        GetAccountInfoQuery::new()
    }
}

/// Helper struct for `getProgramAccounts`
pub struct GetProgramAccounts<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
    min_context_slot: Option<u64>,
}

impl<'q> GetProgramAccounts<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetProgramAccounts {
            base58_public_key: "",
            commitment: "",
            min_context_slot: Option::default(),
        }
    }

    /// Add a base58 public key
    pub fn add_public_key(mut self, base58_public_key: &'q str) -> Self {
        self.base58_public_key = base58_public_key;

        self
    }

    /// Add the commitment level
    pub fn add_commitment(mut self, commitment: &'q str) -> Self {
        self.commitment = commitment;

        self
    }

    /// Add the minimum context slot
    pub fn add_min_context_slot(mut self, min_context_slot: Option<u64>) -> Self {
        self.min_context_slot = min_context_slot;

        self
    }

    /// Build the SQL query
    pub async fn query(self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let commitment = commitment.queryable();
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        if let Some(min_context_slot) = self.min_context_slot {
            let slot = min_context_slot as i64;

            let rows = pg_client.query("
                SELECT DISTINCT on(account_write.pubkey) 
                    account_write.pubkey, account_write.owner, account_write.lamports, account_write.executable, account_write.rent_epoch, account_write.data
                FROM account_write
                WHERE
                    slot >= (SELECT MIN($1) FROM slot WHERE slot.status::VARCHAR = $2::TEXT)
                AND owner = $3::TEXT
                ORDER BY account_write.pubkey, account_write.slot;
            ",
            &[&slot, &commitment, &owner]).await?;

            Ok(rows)
        } else {
            let rows = pg_client.query(
                "
            SELECT DISTINCT on(account_write.pubkey) 
            account_write.pubkey, account_write.owner, account_write.lamports, account_write.executable, account_write.rent_epoch, account_write.data
            FROM account_write
                WHERE
                    rooted = true
                AND owner = $1::TEXT
                ORDER BY account_write.pubkey, account_write.slot DESC;
                ",
                &[ &owner]
            ).await?;

            Ok(rows)
        }
    }
}

impl<'q> Default for GetProgramAccounts<'q> {
    fn default() -> Self {
        GetProgramAccounts::new()
    }
}

/// Get the current slot by querying the `MAX` slot from the database
#[derive(Debug)]
pub struct CurrentSlot {
    /// The commitment to use to get the max slot
    pub commitment: Commitment,
}

impl Default for CurrentSlot {
    fn default() -> Self {
        CurrentSlot {
            commitment: Commitment::Finalized,
        }
    }
}

impl CurrentSlot {
    /// Instantiate a new structure
    pub fn new() -> Self {
        CurrentSlot::default()
    }

    /// Change the commitment level for the query
    pub fn add_commitment(mut self, commitment: Commitment) -> Self {
        self.commitment = commitment;

        self
    }

    /// Run the query in the database and deserialize it to [Self]
    pub async fn query(self) -> ProxyResult<Context> {
        let commitment = self.commitment.queryable();

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let row = pg_client
            .query_one(
                "
            SELECT MAX(slot) FROM slot WHERE status::VARCHAR = $1::TEXT;
            ",
                &[&commitment],
            )
            .await?;

        let context: Context = row.into();

        Ok(context)
    }
}
