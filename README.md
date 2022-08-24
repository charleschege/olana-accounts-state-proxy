### solana-accounts-proxy
This crate is a proxy server that handles fetching account information on a public key for Solana RPC requests. It handles `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts` RPC methods. It speeds up RPC requests by fetching information from a PostgreSQL server connected to a Solana RPC node as a `Geyser Plugin`.

##### Making a request to this server
The server only accepts `POST` requests and will only process supported RPC methods `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts`.
The body must be valid JSON in the same format as JSON data sent to a Solana RPC node in the format
```json
{ 
    jsonrpc: String, 
    id: u8, 
    method: String, 
    params: JsonValue
}
```
 where the `JsonValue` can be any Rust supported primitive type.

The binary will listen on default network socket `https://0.0.0.0:1024`.

##### StatusCodes and JSON Data
- `200`: The HTTP Status Code `200` will show that the POST request was processed successfully and that the JSON body was parsed successfully, the request was then made to the data store and a response was generated successfully as JSON encoding of the [RpcResponse] struct.

- `400`: The HTTP Status Code `400` will show that the POST request was processed successfully but the JSON data is invalid. A JSON encoded String of `RpcResponse<JsonError>` from structs [RpcResponse] and [JsonError] is returned to the user.