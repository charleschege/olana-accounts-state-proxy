use crate::{Filter, GetProgramAccounts, ProxyError, ProxyResult};
use tokio_postgres::{types::ToSql, Row};

impl<'q, 'a> GetProgramAccounts<'q> {
    /// `gPA` accounts with commitment level `Confirmed` and `mint`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn confirmed_with_memcmp(&self) -> ProxyResult<Vec<Row>> {
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let mut query = "SELECT DISTINCT on(pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts
        WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed') 
        AND owner = $1::TEXT".to_owned();

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
                Filter::DataSize(_data_size) => {
                    //params.push(&sizes[cnt1]);
                }
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

            dbg!(&query);

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
    pub async fn confirmed_memcmp_and_data_slice(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let owner = self.base58_public_key;

        let data_slice = match self.data_slice {
            Some(data_slice) => data_slice,
            None => {
                return Err(ProxyError::Client(
                    "The `dataSlice` field is required to perform this query".to_owned(),
                ))
            }
        };
        let data_slice_offset = data_slice.offset as u32 + 1;
        let data_slice_length = data_slice.length as i32;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let mut query = "
        SELECT DISTINCT on(accounts.pubkey) 
            pubkey, lamports, owner, executable, rent_epoch, data, SUBSTRING(data, $1, $2) 
        FROM accounts
        WHERE slot = (SELECT MAX(slot) FROM slots WHERE status = 'confirmed') 
        AND owner = $3::TEXT "
            .to_owned();

        let mut memcmps_temp = Vec::<u32>::new();
        let mut bytes = Vec::<u8>::new();

        let mut params: Vec<&(dyn ToSql + Sync)> =
            vec![&data_slice_offset, &data_slice_length, &owner];

        let mut index = 1usize;

        let mut data_size = Option::<i64>::None;

        for filter in self.filters.as_ref().unwrap() {
            let filter = filter.clone();

            match filter {
                Filter::DataSize(client_data_size) => {
                    index += 1;

                    data_size.replace(client_data_size as i64);
                }
                Filter::Memcmp(memcmp_data) => {
                    let offset = memcmp_data.offset as u32 + 1;
                    let client_bytes = memcmp_data.decode()?;
                    let bytes_len = client_bytes.len() as u32;
                    bytes.extend_from_slice(&client_bytes);

                    query += &format!(
                        " AND substring(data,${},${}) = ${}",
                        index + 1,
                        index + 2,
                        index + 3
                    );

                    index += 3;

                    memcmps_temp.push(offset);
                    memcmps_temp.push(bytes_len);
                }
            }
        }

        if let Some(data_size_exists) = data_size {
            query += &format!(" AND length(data) = ${}", params.len() + 1);

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
