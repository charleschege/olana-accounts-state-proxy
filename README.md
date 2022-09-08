### solana-accounts-proxy
This crate is a proxy server that handles fetching account information on a public key for Solana RPC requests. It handles `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts` RPC methods. It speeds up RPC requests by fetching information from a PostgreSQL server connected to a Solana RPC node as a `Geyser Plugin`.

##### Running the binary

The binary expects a path to the ProxyConfig.toml file to run.

##### The `ProxyConfig.toml` file

```toml
[socket]
ip = "127.0.0.1" #optional
port = 4000 #optional

[postgres]
username = "example" #required
password = "example_password" #required
db_ip = "localhost" #required
db_name = "solana_txs" #required
max_connections = 100 #optional
min_connections = 5 #optional
connect_timeout = 8 #optional
idle_timeout = 8 #optional
max_lifetime = 8 #optional
```

This file has two sections, the `[socket]` section and the `[postgres]`

The `[socket]` section contains the `ip` part which configures the IP address of the server and the `port` which server's HTTP listening port. Both of these fields are mandatory.

The `[postgres]` section covers the settings  for the Postgres connection the server uses to connect to the underlying data store.

- `username` - The username of a Postgres database URL. This field is mandatory.
- `password` -  The password of a Postgres database URL. This field is mandatory.
- `db_ip`  - The host name of a Postgres database URL which is mostly `localhost`. This field is mandatory.
- `db_name` - The name of the database to connect to. The field is mandatory.
- `max_connections` - The maximum number of connections the database connection should use. The field is optional.
- `min_connections` - The maximum number of connections the database connection should use. The field is optional.
- `connect_timeout` - Set the timeout duration when acquiring a connection. The field is optional.
- `idle_timeout` - Set the idle duration before closing a connection. The field is optional.
- `max_lifetime` - Set the maximum lifetime of individual connections. The field is optional.

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
$ cargo run --release -- /path/to/ProxyConfig.toml/file
```

To compile and run the crate with logging enabled (suitable for debug builds)

```sh
cargo run --release --features log_with_tracing -- /path/to/ProxyConfig.toml/file
```

##### Extra compile time features

- `dangerous_debug` - The Postgres database name, password and username are protected from being accidentally logged or copied in memory and are automatically zeroed out from memory when they are dropped/out of scope. Therefore, debugging the `PostgresConfig` struct would result in seeing `REDACTED[...]` string output rather than the actual password or username. Using the `dangerous_debug` feature allows you to see the contents of the password, username and database name when compiling in debug mode.
