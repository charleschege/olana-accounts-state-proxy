use crate::{Commitment, DataSlice, Filter, ProxyResult};

mod with_confirmed;
pub use with_confirmed::*;

mod with_processed;
pub use with_processed::*;

mod with_finalized;
pub use with_finalized::*;

/// Helper struct for `getProgramAccounts`
#[derive(Debug)]
pub struct GetProgramAccounts<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
    min_context_slot: Option<u64>,
    data_slice: Option<DataSlice>,
    filters: Option<Vec<Filter>>,
}

impl<'q> GetProgramAccounts<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetProgramAccounts {
            base58_public_key: "",
            commitment: "",
            min_context_slot: Option::default(),
            data_slice: Option::default(),
            filters: Option::default(),
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

    /// Add the data slice
    pub fn add_data_slice(mut self, data_slice: Option<DataSlice>) -> Self {
        self.data_slice = data_slice;

        self
    }

    /// Add the filters for the query
    pub fn add_filters(mut self, filters: Option<Vec<Filter>>) -> Self {
        self.filters = filters;

        self
    }

    /// Executor for the queries
    pub async fn load_data(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();

        // Check if only basic queries are supported
        if self.filters.is_none() && self.data_slice.is_none() && self.min_context_slot.is_none() {
            //TODO `minContextSlot`
            self.basic_with_commitment().await
        }
        // Queries with no `dataSlice` field
        else if self.filters.is_some() && self.data_slice.is_none() {
            if commitment == Commitment::Processed {
                self.processed_with_memcmp().await
            } else if commitment == Commitment::Confirmed {
                self.confirmed_with_memcmp().await
            } else {
                self.finalized_with_memcmp().await
            }
        }
        // Queries with `dataSlice` field
        else if self.filters.is_some() && self.data_slice.is_some() {
            if commitment == Commitment::Processed {
                self.processed_memcmp_and_data_slice().await
            } else if commitment == Commitment::Confirmed {
                self.confirmed_memcmp_and_data_slice().await
            } else {
                self.finalized_memcmp_and_data_slice().await
            }
        } else {
            todo!()
        }
    }

    /// `gPA` accounts with commitment level and an `owner`
    pub async fn basic_with_commitment(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        if commitment == Commitment::Processed {
            let rows = pg_client.query("
            SELECT DISTINCT ON(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts 
            WHERE (slot = (SELECT MAX(slot) FROM slot WHERE slots.status = 'confirmed' OR slot.status='processed'))
            AND owner = $1::TEXT
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[&owner]).await?;

            crate::row_data_size_info(rows.len());

            Ok(rows)
        } else if commitment == Commitment::Confirmed {
            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts 
            WHERE (slot = (SELECT MAX(slot) FROM slot WHERE slots.status = 'confirmed') )
            AND owner = $1::TEXT
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[&owner]).await?;

            crate::row_data_size_info(rows.len());

            Ok(rows)
        } else {
            let rows = pg_client
                .query(
                    "
                SELECT DISTINCT on(accounts.pubkey)
                    pubkey, lamports, owner, executable, rent_epoch, data
                FROM accounts
                WHERE
                    slot = (SELECT MAX(slot) FROM slots WHERE status = 'finalized')
                AND owner = $1::TEXT
                ORDER BY accounts.pubkey, accounts.slot DESC;
                ",
                    &[&owner],
                )
                .await?;

            crate::row_data_size_info(rows.len());

            Ok(rows)
        }
    }
}

impl<'q> Default for GetProgramAccounts<'q> {
    fn default() -> Self {
        GetProgramAccounts::new()
    }
}
