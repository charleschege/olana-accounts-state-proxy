use crate::{Commitment, DataSlice, Filter, ProxyError, ProxyResult};
use tokio_postgres::{types::ToSql, Row};

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
        if self.data_slice.is_some() {
            self.with_memcmp_and_data_slice().await
        } else if self.data_slice.is_some() {
            self.with_memcmp().await
        } else {
            todo!()
        }
    }

    /// `gPA` accounts with commitment level `Confirmed` and `mint`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn with_memcmp(&self) -> ProxyResult<Vec<Row>> {
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let mut query = "SELECT DISTINCT on(pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts ".to_owned();

        let commitment: Commitment = self.commitment.into();

        match commitment {
            Commitment::Processed => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed' OR status='processed')";
            }
            Commitment::Confirmed => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed')";
            }
            Commitment::Finalized => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'finalized')";
            }
        }

        query += "AND owner = $1::TEXT";

        let mut sizes: Vec<i32> = vec![];
        let mut bytes = vec![];
        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::Memcmp(memcmp_data) => {
                    let decoded_bytes = memcmp_data.decode()?;

                    sizes.push(memcmp_data.offset as i32 + 1);
                    sizes.push(decoded_bytes.len() as i32);
                    bytes.push(decoded_bytes);
                }
                _ => (),
            }
        }

        let mut data_size = Option::<i32>::None;

        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::DataSize(client_data_size) => {
                    data_size.replace(client_data_size as i32);
                }
                _ => (),
            }
        }

        let mut params: Vec<&(dyn ToSql + Sync)> = vec![&owner];
        let mut cnt1 = 0;
        let mut cnt2 = 0;
        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::DataSize(_data_size) => {}
                Filter::Memcmp(_memcmp_data) => {
                    let len = params.len();
                    query += &format!(
                        " AND substring(data,${},${}) = ${}",
                        len + 1,
                        len + 2,
                        len + 3
                    );

                    params.push(&sizes[cnt1]);
                    cnt1 += 1;
                    params.push(&sizes[cnt1]);
                    cnt1 += 1;
                    params.push(&bytes[cnt2]);
                    cnt2 += 1;
                }
            }
        }

        if let Some(data_size_exists) = data_size {
            query += &format!(" AND length(data) = ${};", params.len() + 1);

            params.push(&data_size_exists);

            let rows = pg_client.query(&query, &params).await?;

            Ok(rows)
        } else {
            query += ";";
            let rows = pg_client.query(&query, &params).await?;

            Ok(rows)
        }
    }

    /// `gPA` accounts with `Commitment` level, `Filters` and `dataSlice`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn with_memcmp_and_data_slice(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let owner = self.base58_public_key;

        let data_slice = match self.data_slice {
            Some(data_slice) => data_slice,
            None => {
                //FIXME Remove this error possibly by combining both the `with_memcmp` and `with_memcmp_and_data_slice` methods but you have to deal with lifetimes
                return Err(ProxyError::Client(
                    "The `dataSlice` field is required to perform this query".to_owned(),
                ));
            }
        };
        let data_slice_offset = data_slice.offset as i32 + 1;
        let data_slice_length = data_slice.length as i32;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let mut query = "
        SELECT DISTINCT on(accounts.pubkey) 
            pubkey, lamports, owner, executable, rent_epoch, data, SUBSTRING(data, $1, $2) 
        FROM accounts "
            .to_owned();
        let commitment: Commitment = self.commitment.into();

        match commitment {
            Commitment::Processed => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed' OR status='processed')";
            }
            Commitment::Confirmed => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed')";
            }
            Commitment::Finalized => {
                query += "WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'finalized')";
            }
        }

        query += "AND owner = $3::TEXT ";

        let mut sizes: Vec<i32> = vec![];
        let mut bytes = vec![];
        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::Memcmp(memcmp_data) => {
                    let decoded_bytes = memcmp_data.decode()?;

                    sizes.push(memcmp_data.offset as i32 + 1);
                    sizes.push(decoded_bytes.len() as i32);
                    bytes.push(decoded_bytes);
                }
                _ => (),
            }
        }

        let mut data_size = Option::<i32>::None;

        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::DataSize(client_data_size) => {
                    data_size.replace(client_data_size as i32);
                }
                _ => (),
            }
        }

        let mut params: Vec<&(dyn ToSql + Sync)> =
            vec![&data_slice_offset, &data_slice_length, &owner];
        let mut cnt1 = 0;
        let mut cnt2 = 0;
        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();
            match filter {
                Filter::DataSize(_data_size) => {}
                Filter::Memcmp(_memcmp_data) => {
                    let len = params.len();
                    query += &format!(
                        " AND substring(data,${},${}) = ${}",
                        len + 1,
                        len + 2,
                        len + 3
                    );

                    params.push(&sizes[cnt1]);
                    cnt1 += 1;
                    params.push(&sizes[cnt1]);
                    cnt1 += 1;
                    params.push(&bytes[cnt2]);
                    cnt2 += 1;
                }
            }
        }

        if let Some(data_size_exists) = data_size {
            query += &format!(" AND length(data) = ${};", params.len() + 1);

            params.push(&data_size_exists);

            let rows = pg_client.query(&query, &params).await?;

            Ok(rows)
        } else {
            query += ";";
            let rows = pg_client.query(&query, &params).await?;

            Ok(rows)
        }
    }
}

impl<'q> Default for GetProgramAccounts<'q> {
    fn default() -> Self {
        GetProgramAccounts::new()
    }
}
