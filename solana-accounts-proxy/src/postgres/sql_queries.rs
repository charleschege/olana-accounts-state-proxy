/// Helper struct to create the query for `getAccountInfo` using the builder pattern
pub struct GetAccountInfoQuery<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
}

impl<'q> GetAccountInfoQuery<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetAccountInfoQuery {
            base58_public_key: "",
            commitment: "",
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

    /// Build the SQL query
    pub fn query(self) -> String {
        let mut query = String::new();

        query.push_str(
            "SELECT 
            account_write.slot,
            account_write.data,
            account_write.executable,
            account_write.owner,
            account_write.lamports,
            account_write.rent_epoch
        FROM account_write WHERE pubkey = '",
        );
        query.push_str(self.base58_public_key);
        query.push_str("';");

        query
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
}

impl<'q> GetProgramAccounts<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetProgramAccounts {
            base58_public_key: "",
            commitment: "",
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
}

impl<'q> Default for GetProgramAccounts<'q> {
    fn default() -> Self {
        GetProgramAccounts::new()
    }
}
