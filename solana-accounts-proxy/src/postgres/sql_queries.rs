use crate::{
    Commitment, Context, DataSlice, Encoding, Filter, GetAccountInfoRow, ProxyError, ProxyResult,
};

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

    /// `gPA` accounts with commitment level and an `owner`
    pub async fn basic_with_commitment(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        if commitment == Commitment::Processed {
            let rows = pg_client.query("
            SELECT DISTINCT ON(account_write.pubkey) account_write.pubkey FROM account_write 
            WHERE (rooted = TRUE OR slot = (SELECT MAX(slot) FROM slot WHERE slot.status = 'Confirmed' OR slot.status='Processed'))
            AND owner = $1::TEXT
            ORDER BY account_write.pubkey, account_write.slot DESC;
            ", &[&owner]).await?;

            Ok(rows)
        } else if commitment == Commitment::Confirmed {
            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) account_write.pubkey FROM account_write 
            WHERE (rooted = TRUE OR slot = (SELECT MAX(slot) FROM slot WHERE slot.status = 'Confirmed') )
            AND owner = $1::TEXT
            ORDER BY account_write.pubkey, account_write.slot DESC;
            ", &[&owner]).await?;

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
                &[&owner]
            ).await?;

            Ok(rows)
        }
    }

    /// `gPA` accounts with commitment level and `mint`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn with_memcmp(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let memcmps = match self.filters.clone() {
            Some(filters) => Filter::memcmps(filters)?,
            None => {
                return Err(ProxyError::Client(
                    "Expected a `Filter` with `MemCmp` to build this query".to_owned(),
                ))
            }
        };

        let data_size = match &self.filters {
            Some(filters) => Filter::data_size(&filters)? as i64,
            None => {
                return Err(ProxyError::Client(
                    "Expected a `Filter` with a `dataSize` to build this query".to_owned(),
                ))
            }
        };

         if memcmps.len() == 2 {
            // Get values for the first MemCmp
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i64;
            offset1 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len1 = memcmp_bytes1.len() as i64;
            let mut memcmp_data_hex1 = String::from("\\x");
            memcmp_data_hex1.push_str(&hex::encode(&memcmp_bytes1));

            // Get values for second MemCmp
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i64;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len2 = memcmp_bytes2.len() as i64;
            let mut memcmp_data_hex2 = String::from("\\x");
            memcmp_data_hex2.push_str(&hex::encode(&memcmp_bytes2));

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) * FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4::TEXT
            AND substring(data,$5,$6) = $7::TEXT
            AND length(data) = $8                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_data_hex1, 
                &offset2, &offset_bytes_len2, &memcmp_data_hex2, 
                &data_size]).await?;

            Ok(rows)
        } else if memcmps.len() == 3 {
            // Get values for the first MemCmp at index `2` 
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i64;
            offset1 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len1 = memcmp_bytes1.len() as i64;
            let mut memcmp_data_hex1 = String::from("\\x");
            memcmp_data_hex1.push_str(&hex::encode(&memcmp_bytes1));

            // Get values for second MemCmp at index `3` of 
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i64;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len2 = memcmp_bytes2.len() as i64;
            let mut memcmp_data_hex2 = String::from("\\x");
            memcmp_data_hex2.push_str(&hex::encode(&memcmp_bytes2));

            // Get values for the first MemCmp at index `4` 
            let memcmp3 = memcmps[1].clone();
            let memcmp_bytes3 = Encoding::Base58.decode(memcmp3.bytes.as_bytes())?;
            let mut offset3 = memcmp3.offset as i64;
            offset3 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len3 = memcmp_bytes3.len() as i64;
            let mut memcmp_data_hex3 = String::from("\\x");
            memcmp_data_hex3.push_str(&hex::encode(&memcmp_bytes3));

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) * FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4::TEXT
            AND substring(data,$5,$6) = $7::TEXT
            AND substring(data,$8,$9) = $10::TEXT
            AND length(data) = $11                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_data_hex1, 
                &offset2, &offset_bytes_len2, &memcmp_data_hex2,
                &offset3, &offset_bytes_len3, &memcmp_data_hex3, 
                &data_size]).await?;

            Ok(rows)
        }else {
            let memcmp1 = memcmps[0].clone();

            let memcmp_bytes = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset = memcmp1.offset as i64;
            offset += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len = memcmp_bytes.len() as i64;

            let mut memcmp_data_hex = String::from("\\x");
            memcmp_data_hex.push_str(&hex::encode(&memcmp_bytes));

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) * FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4::TEXT
            AND length(data) = $5                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[&owner, &offset, &offset_bytes_len, &memcmp_data_hex, &data_size]).await?;

            Ok(rows)
        }
    }


    /// `gPA` accounts with `Commitment` level, `Filters` and `dataSlice`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn with_memcmp_and_data_slice(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let owner = self.base58_public_key;
        let data_slice = match self.data_slice {
            Some(data_slice) => data_slice,
            None => return Err(ProxyError::Client("The `dataSlice` field is required to perform this query".to_owned()))
        };

        let data_slice_offset = data_slice.offset + 1 ;
        let data_slice_offset = data_slice_offset as i64;
        let data_slice_length = data_slice.length as i64;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let memcmps = match self.filters.clone() {
            Some(filters) => Filter::memcmps(filters)?,
            None => {
                return Err(ProxyError::Client(
                    "Expected a `Filter` with `MemCmp` to build this query".to_owned(),
                ))
            }
        };

        let data_size = match &self.filters {
            Some(filters) => Filter::data_size(&filters)? as i64,
            None => {
                return Err(ProxyError::Client(
                    "Expected a `Filter` with a `dataSize` to build this query".to_owned(),
                ))
            }
        };

         if memcmps.len() == 2 {
            // Get values for the first MemCmp
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i64;
            offset1 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len1 = memcmp_bytes1.len() as i64;
            let mut memcmp_data_hex1 = String::from("\\x");
            memcmp_data_hex1.push_str(&hex::encode(&memcmp_bytes1));

            // Get values for second MemCmp
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i64;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len2 = memcmp_bytes2.len() as i64;
            let mut memcmp_data_hex2 = String::from("\\x");
            memcmp_data_hex2.push_str(&hex::encode(&memcmp_bytes2));
            

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) pubkey, slot, owner, lamports, executable, rent_epoch, SUBSTRING(data, $1, $2) FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6::TEXT
            AND substring(data,$7,$8) = $9::TEXT
            AND length(data) = $10                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[
                &data_slice_offset,
                &data_slice_length,
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_data_hex1, 
                &offset2, &offset_bytes_len2, &memcmp_data_hex2, 
                &data_size]).await?;

            Ok(rows)
        } else if memcmps.len() == 3 {
            // Get values for the first MemCmp at index `2` 
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i64;
            offset1 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len1 = memcmp_bytes1.len() as i64;
            let mut memcmp_data_hex1 = String::from("\\x");
            memcmp_data_hex1.push_str(&hex::encode(&memcmp_bytes1));

            // Get values for second MemCmp at index `3` of 
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i64;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len2 = memcmp_bytes2.len() as i64;
            let mut memcmp_data_hex2 = String::from("\\x");
            memcmp_data_hex2.push_str(&hex::encode(&memcmp_bytes2));

            // Get values for the first MemCmp at index `4` 
            let memcmp3 = memcmps[1].clone();
            let memcmp_bytes3 = Encoding::Base58.decode(memcmp3.bytes.as_bytes())?;
            let mut offset3 = memcmp3.offset as i64;
            offset3 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len3 = memcmp_bytes3.len() as i64;
            let mut memcmp_data_hex3 = String::from("\\x");
            memcmp_data_hex3.push_str(&hex::encode(&memcmp_bytes3));

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) pubkey, slot, owner, lamports, executable, rent_epoch, SUBSTRING(data, $1, $2) FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6::TEXT
            AND substring(data,$7,$8) = $9::TEXT
            AND substring(data,$10,$11) = $12::TEXT
            AND length(data) = $13                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[
                &data_slice_offset,
                &data_slice_length,
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_data_hex1, 
                &offset2, &offset_bytes_len2, &memcmp_data_hex2,
                &offset3, &offset_bytes_len3, &memcmp_data_hex3, 
                &data_size]).await?;

            Ok(rows)
        }else {
            let memcmp1 = memcmps[0].clone();

            let memcmp_bytes = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset = memcmp1.offset as i64;
            offset += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `1`
            let offset_bytes_len = memcmp_bytes.len() as i64;

            let mut memcmp_data_hex = String::from("\\x");
            memcmp_data_hex.push_str(&hex::encode(&memcmp_bytes));

            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) pubkey, slot, owner, lamports, executable, rent_epoch, SUBSTRING(data, $1, $2) FROM account_write
            WHERE                         
                rooted = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6::TEXT
            AND length(data) = $7                                                      
            ORDER BY account_write.pubkey, account_write.slot DESC, account_write.write_version DESC;
            ", &[
                &data_slice_offset, &data_slice_length, &owner, &offset, &offset_bytes_len, &memcmp_data_hex, &data_size]).await?;

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
