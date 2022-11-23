use crate::{
      GetProgramAccounts,  Encoding, Filter,  ProxyError, ProxyResult,
};

impl<'q> GetProgramAccounts<'q> {
    
    /// `gPA` accounts with commitment level `Finalized` and `mint`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn finalized_with_memcmp(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
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
            Some(filters) => Filter::data_size(&filters)? as i32,
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
            let mut offset1 = memcmp1.offset as i32;
            offset1 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len1 = memcmp_bytes1.len() as i32;

            // Get values for second MemCmp
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i32;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len2 = memcmp_bytes2.len() as i32;

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts
            WHERE                         
                finalized = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4
            AND substring(data,$5,$6) = $7
            AND length(data) = $8                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_bytes1, 
                &offset2, &offset_bytes_len2, &memcmp_bytes2, 
                &data_size]).await?;
                
                crate::row_data_size_info(rows.len());

            Ok(rows)
        } else if memcmps.len() == 3 {
            // Get values for the first MemCmp at index `2` 
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i32;
            offset1 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len1 = memcmp_bytes1.len() as i32;

            // Get values for second MemCmp at index `3` of 
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i32;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len2 = memcmp_bytes2.len() as i32;

            // Get values for the first MemCmp at index `4` 
            let memcmp3 = memcmps[1].clone();
            let memcmp_bytes3 = Encoding::Base58.decode(memcmp3.bytes.as_bytes())?;
            let mut offset3 = memcmp3.offset as i32;
            offset3 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len3 = memcmp_bytes3.len() as i32;

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM  accounts
            WHERE                         
                finalized = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4
            AND substring(data,$5,$6) = $7
            AND substring(data,$8,$9) = $10
            AND length(data) = $11                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_bytes1, 
                &offset2, &offset_bytes_len2, &memcmp_bytes2,
                &offset3, &offset_bytes_len3, &memcmp_bytes3, 
                &data_size]).await?;
                
                crate::row_data_size_info(rows.len());

            Ok(rows)
        }else {
            let memcmp1 = memcmps[0].clone();

            let memcmp_bytes = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset = memcmp1.offset as i32;
            offset += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len = memcmp_bytes.len() as i32;

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data FROM accounts
            WHERE                         
                finalized = TRUE
            AND owner = $1::TEXT 
            AND substring(data,$2,$3) = $4
            AND length(data) = $5                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[&owner, &offset, &offset_bytes_len, &memcmp_bytes, &data_size]).await?;
            
            crate::row_data_size_info(rows.len());

            Ok(rows)
        }
    }

    /// `gPA` accounts with `Commitment` level, `Filters` and `dataSlice`
    // substring(data, {1}, {2}), memcmp.offset+1, len(memcmp.bytes)
    pub async fn finalized_memcmp_and_data_slice(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let owner = self.base58_public_key;
        let data_slice = match self.data_slice {
            Some(data_slice) => data_slice,
            None => return Err(ProxyError::Client("The `dataSlice` field is required to perform this query".to_owned()))
        };

        let data_slice_offset = data_slice.offset + 1 ;
        let data_slice_offset = data_slice_offset as i32;
        let data_slice_length = data_slice.length as i32;

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
            Some(filters) => Filter::data_size(&filters)? as i32,
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
            let mut offset1 = memcmp1.offset as i32;
            offset1 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len1 = memcmp_bytes1.len() as i32;

            // Get values for second MemCmp
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i32;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len2 = memcmp_bytes2.len() as i32;
            

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data, SUBSTRING(data, $1, $2) FROM accounts
            WHERE                         
                finalized = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6
            AND substring(data,$7,$8) = $9
            AND length(data) = $10                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[
                &data_slice_offset,
                &data_slice_length,
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_bytes1, 
                &offset2, &offset_bytes_len2, &memcmp_bytes2, 
                &data_size]).await?;
                
                crate::row_data_size_info(rows.len());

            Ok(rows)
        } else if memcmps.len() == 3 {
            // Get values for the first MemCmp at index `2` 
            let memcmp1 = memcmps[0].clone();
            let memcmp_bytes1 = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset1 = memcmp1.offset as i32;
            offset1 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len1 = memcmp_bytes1.len() as i32;

            // Get values for second MemCmp at index `3` of 
            let memcmp2 = memcmps[1].clone();
            let memcmp_bytes2 = Encoding::Base58.decode(memcmp2.bytes.as_bytes())?;
            let mut offset2 = memcmp2.offset as i32;
            offset2 += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len2 = memcmp_bytes2.len() as i32;

            // Get values for the first MemCmp at index `4` 
            let memcmp3 = memcmps[1].clone();
            let memcmp_bytes3 = Encoding::Base58.decode(memcmp3.bytes.as_bytes())?;
            let mut offset3 = memcmp3.offset as i32;
            offset3 += 1 ; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len3 = memcmp_bytes3.len() as i32;

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data, SUBSTRING(data, $1, $2) FROM accounts
            WHERE                         
                finalized = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6
            AND substring(data,$7,$8) = $9
            AND substring(data,$10,$11) = $12
            AND length(data) = $13                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[
                &data_slice_offset,
                &data_slice_length,
                &owner, 
                &offset1, &offset_bytes_len1, &memcmp_bytes1, 
                &offset2, &offset_bytes_len2, &memcmp_bytes2,
                &offset3, &offset_bytes_len3, &memcmp_bytes3, 
                &data_size]).await?;
                
                crate::row_data_size_info(rows.len());

            Ok(rows)
        }else {
            let memcmp1 = memcmps[0].clone();

            let memcmp_bytes = Encoding::Base58.decode(memcmp1.bytes.as_bytes())?;
            let mut offset = memcmp1.offset as i32;
            offset += 1; // PostgreSQL starts offset from `1` but Solana SPL Token account offset starts from `0`
            let offset_bytes_len = memcmp_bytes.len() as i32;

            let rows = pg_client.query("
            SELECT DISTINCT on(accounts.pubkey) pubkey, lamports, owner, executable, rent_epoch, data, SUBSTRING(data, $1, $2) FROM accounts
            WHERE                         
                finalized = TRUE
            AND owner = $3::TEXT 
            AND substring(data,$4,$5) = $6
            AND length(data) = $7                                                      
            ORDER BY accounts.pubkey, accounts.slot DESC;
            ", &[
                &data_slice_offset, &data_slice_length, &owner, &offset, &offset_bytes_len, &memcmp_bytes, &data_size]).await?;
                
                crate::row_data_size_info(rows.len());

            Ok(rows)
        }
    }
}

