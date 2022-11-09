use core::fmt;
use jsonrpsee::core::{Error as JsonrpseeError, RpcResult};
use serde::Deserialize;

/// Holds and ed25519 public key for a Solana program or account
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PubKey(pub [u8; 32]);

impl PubKey {
    /// Converts a base58 formatted String into a [PubKey]
    pub fn parse(value: &str) -> RpcResult<PubKey> {
        let decoded_bytes = match bs58::decode(&value).into_vec() {
            Ok(value) => value,
            Err(_) => {
                return Err(JsonrpseeError::Custom(
                    "The encoded public key is not valid Base58 format".to_owned(),
                ))
            }
        };

        let decoded_bytes_len = decoded_bytes.len();

        let public_key: [u8; 32] = match decoded_bytes.try_into() {
            Ok(public_key) => public_key,
            Err(_) => {
                let mut error = String::new();
                error.push_str("The encoded public key was decoded properly as bytes but it has an invalid length of `");
                error.push_str(decoded_bytes_len.to_string().as_str());
                error.push_str("` bytes instead of `32 bytes`.");

                return Err(JsonrpseeError::Custom(error));
            }
        };

        Ok(PubKey(public_key))
    }
}

impl fmt::Debug for PubKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bs58_public_key = bs58::encode(&self.0).into_string();

        f.debug_tuple("PubKey").field(&bs58_public_key).finish()
    }
}

/// Parse the parameters from the JSON data
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// The commitment level of a block
    pub commitment: Option<Commitment>,
    /// How the RPC should encode the data when transmitting to a client
    pub encoding: Option<Encoding>,
    /// Limits the account data  using the provided offset and length
    pub data_slice: Option<DataSlice>,
    /// The minimum slot the request can be evaluated at
    pub min_context_slot: Option<u64>,
    /// Filters to use on the results
    //pub filters: Option<Vec<Filter>>,
    pub filters: Option<(DataSize, MemCmp)>,
    /// wrap the result in an RpcResponse JSON object.
    pub with_context: Option<bool>,
}

/// Which format the proxy server should use when transmitting a response data to a client
#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Encoding {
    /// Use Base58 encoding
    #[serde(rename = "base58")]
    Base58,
    /// Use Base64 encoding
    #[serde(rename = "base64")]
    Base64,
    /// Use Base64 encoding with zstd compression
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
    /// Use jsonParsed encoding with the available serializer
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

impl Encoding {
    /// Check which encoding format to use on the data field
    pub fn get_encoding(parameters: Option<&Parameters>) -> Encoding {
        if let Some(parameters) = parameters {
            match &parameters.encoding {
                None => Encoding::Base58,
                Some(encoding) => *encoding,
            }
        } else {
            Encoding::Base58
        }
    }

    /// Encode data to the chosen format
    pub fn encode(&self, data: &[u8]) -> RpcResult<String> {
        match self {
            Self::Base58 => {
                if data.len() > 129 {
                    // FIXME Use correct JSON response code of `-32001`
                    return Err(JsonrpseeError::ResourceNameAlreadyTaken("Encoded binary (base 58) data should be less than 128 bytes, please use Base64 encoding."));
                }

                tracing::debug!("ENCODING DATA CHUNK AS Base58");
                let data = bs58::encode(data).into_string();
                tracing::debug!("FINISHED ENCODING DATA CHUNK AS Base58");

                Ok(data)
            }
            Self::Base64 => {
                tracing::debug!("ENCODING DATA CHUNK AS Base64");
                let data = base64::encode(data);
                tracing::debug!("FINISHED ENCODING DATA CHUNK AS Base64");

                Ok(data)
            }
            Self::Base64Zstd => {
                tracing::debug!("ENCODING DATA CHUNK AS Base64+zstd");

                let mut buffer = data.to_vec();
                let encoder = match zstd::Encoder::new(&mut buffer, 3) {
                    Ok(data) => data,
                    Err(error) => return Err(JsonrpseeError::Custom(error.to_string())),
                };

                match encoder.finish() {
                    Ok(data) => data,
                    Err(error) => return Err(JsonrpseeError::Custom(error.to_string())),
                };

                let data = base64::encode(&buffer);

                tracing::debug!("FINISHED ENCODING DATA CHUNK AS Base64+zstd");

                Ok(data)
            }
            Self::JsonParsed => {
                tracing::debug!("FINISHED ENCODING DATA CHUNK AS JsonParsed");
                let to_json: json::JsonValue = data.into();

                let data = to_json.to_string();

                tracing::debug!("FINISHED ENCODING DATA CHUNK AS JsonParsed");

                Ok(data)
            }
        }
    }

    /// Decode data from a method parameter,
    /// `NOTE:` Only `base64` and `base58` formats, all other formats result in an RPC error.
    pub fn decode(&self, data: &[u8]) -> RpcResult<Vec<u8>> {
        match self {
            Self::Base58 => match bs58::decode(data).into_vec() {
                Ok(decoded_data) => Ok(decoded_data),
                Err(error) => Err(JsonrpseeError::Custom(error.to_string())),
            },
            _ => {
                let mut to_rpc_error = "Unsupported data encoding format `".to_owned();
                to_rpc_error.push_str(self.to_str());
                to_rpc_error.push_str("` for the method.");

                Err(JsonrpseeError::Custom(to_rpc_error.to_string()))
            }
        }
    }

    /// Used to return the encoding type in the JSON response
    pub fn to_str(&self) -> &str {
        match self {
            Self::Base58 => "base58",
            Self::Base64 => "base64",
            Self::Base64Zstd => "base64+zstd",
            Self::JsonParsed => "jsonParsed",
        }
    }
}

/// Whether a block has been confirmed, is being processed or has been finalized
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[derive(postgres_types::ToSql)]
pub enum Commitment {
    /// A block has been processed by the RPC node
    Processed,
    /// A block has been confirmed as valid by the RPC node
    Confirmed,
    /// A block has been finalized and changes cannot be rolled back
    Finalized,
}

impl Commitment {
    /// Convert to a &str usable in the SQL query
    pub fn queryable<'a>(&self) -> &'a str {
        match self {
            Self::Confirmed => "Confirmed",
            Self::Processed => "Processed",
            Self::Finalized => "Rooted",
        }
    }
    /// Returns the commitment level to use when executing the query
    pub fn get_commitment<'a>(parameters: Option<&Parameters>) -> &'a str {
        if let Some(parameters) = parameters {
            match parameters.commitment {
                Some(commitment) => commitment.queryable(),
                None => Commitment::Finalized.queryable(),
            }
        } else {
            Commitment::Finalized.queryable()
        }
    }
}

impl From<&str> for Commitment {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "confirmed" => Self::Confirmed,
            "processed" => Self::Processed,
            _ => Self::Finalized,
        }
    }
}

/// Configures the offset and the length
#[derive(Debug, Deserialize, Default, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSlice {
    /// Limits data to a particular offset
    pub offset: usize,
    /// Limits the data to a particular length
    pub length: usize,
}

/// compares the program account data length with the provided data size
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataSize {
    /// Length to compare
    pub data_size: u64,
}

///  Used to compare a provided series of bytes with program account data
/// at a particular offset.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemCmp {
    /// The data to use for the comparison operation
    pub memcmp: MemCmpData,
}

///  The comparison data of [MemCmp]
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemCmpData {
    /// offset into program account data to start comparison
    pub offset: usize,
    /// data to match, as encoded string
    pub bytes: String,
    /// encoding for filter bytes data, either "base58" or "base64".
    /// Data is limited in size to 128 or fewer decoded bytes.
    pub encoding: Option<Encoding>,
}
