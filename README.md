### solana-accounts-proxy
This crate is a proxy server that handles fetching account information on a public key for Solana RPC requests. It handles `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts` RPC methods. It speeds up RPC requests by fetching information from a PostgreSQL server connected to a Solana RPC node as a `Geyser Plugin`.

##### Running the binary

The proxy server listens at socket `http://0.0.0.0:1024` if the server is run with no extra arguments. 

###### Custom socket settings

1. Running the server with a custom IP address

   ```sh
   $ solana-accounts-proxy -ip 127.0.0.0
   ```

   If a valid IP address is not given, the server exits with an error `server error: AddrParseError`

2. Running with a custom port which is a `u16`

   ```sh
   $ solana-accounts-proxy -port 8000
   ```

   If an invalid `u16` is given for the port, the server exits with a custom error `server error: Int(...`

3. Running with both a custom IP and port

   ```sh
   $ solana-accounts-proxy -ip 127.0.0.0 -port 8000
   ```

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

The binary will listen on default network socket `http://0.0.0.0:1024`.

##### StatusCodes and JSON Data
- `200`: The HTTP Status Code `200` will show that the POST request was processed successfully and that the JSON body was parsed successfully, the request was then made to the data store and a response was generated successfully as JSON encoding of the [RpcResponse] struct.

- `400`: The HTTP Status Code `400` will show that the POST request was processed successfully but the JSON data is invalid. A JSON encoded String of `RpcResponse<JsonError>` from structs [RpcResponse] and [JsonError] is returned to the user.

The JSON body returned by the proxy server is in the same format as that from a Solana RPC node therefore can be parsed using any Solana RPC client.

Errors encountered while parsing the JSON data, checking for supported RPC methods or supported encoding formats are returned to the client with respect to the JSON 2 errors specification - [https://www.jsonrpc.org/specification#error_object](https://www.jsonrpc.org/specification#error_object) . Solana RPC clients use the same specification enabling compatibility.

#### Compiling

To compile and run the crate

```sh
$ cargo run --release
```

To compile and run the crate with logging enabled (suitable for debug builds)

```sh
cargo run --release --features log_with_tracing
```

