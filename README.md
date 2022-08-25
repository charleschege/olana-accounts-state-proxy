### solana-accounts-proxy
This crate is a proxy server that handles fetching account information on a public key for Solana RPC requests. It handles `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts` RPC methods. It speeds up RPC requests by fetching information from a PostgreSQL server connected to a Solana RPC node as a `Geyser Plugin`.

##### Running the binary
To run the proxy server, a configuration file called `ProxyConfig.toml` is required. This configuration file contains the `IP address`, `port` and `TLS keypair` to use for HTTPS connections.

The path to the configuration file is passed as an argument to the server.
```sh
$ solana-accounts-proxy /path-to-directory-containing-config-file
```

If you are compiling and running from source use:
```sh
$ cargo run -- /path-to-directory-containing-config-file
```

##### Structure of the configuration file
```toml
ip = "0.0.0.0" # Ip is a String of the IP address
port = 1024 # port is a u16 of a network port

[tls] #The TLS configuration file
private = "private.rsa" # path to the  RSA private key file
public = "public.pem" # path to the public key file
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

The binary will listen on default network socket `https://0.0.0.0:1024`.

##### StatusCodes and JSON Data
- `200`: The HTTP Status Code `200` will show that the POST request was processed successfully and that the JSON body was parsed successfully, the request was then made to the data store and a response was generated successfully as JSON encoding of the [RpcResponse] struct.

- `400`: The HTTP Status Code `400` will show that the POST request was processed successfully but the JSON data is invalid. A JSON encoded String of `RpcResponse<JsonError>` from structs [RpcResponse] and [JsonError] is returned to the user.