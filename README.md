### solana-accounts-proxy
This crate is a proxy server that handles fetching account information on a public key for Solana RPC requests. It handles `getAccountInfo`, `getMultipleAccounts` and `getProgramAccounts` RPC methods. It speeds up RPC requests by fetching information from a PostgreSQL server connected to a Solana RPC node as a `Geyser Plugin`.

##### Running the binary

The binary expects a path to the ProxyConfig.toml file to run.

##### The `Config.toml` file

```toml
[socket]
ip = "127.0.0.1" # Required field
port = 4000 # Required field

[postgres]
user =  "solana" # Required field
dbname =  "solana" # Required field
host =  "localhost" # Required field
password =  "solana", # Optional field
options =  "foobar", # Optional field
application_name =  "solana_rpc_proxy", # Optional field
port =  5432 # Optional field
connect_timeout =  120,  # Optional field
```

This file has two sections, the `[socket]` section and the `[postgres]`

The `[socket]` section contains the `ip` part which configures the IP address of the server and the `port` which server's HTTP listening port. Both of these fields are mandatory.

The `[postgres]` section covers the settings  for the Postgres connection the server uses to connect to the underlying data store.

- `user` - The user of a Postgres database URL. This field is mandatory.
- `dbname` - The database to connect to. This field is mandatory.
- `host` - The host IP or domain running the database.  This field is mandatory.
- `password` - The password to connect to the database.  This field is optional.
- `options` - The  arguments to pass to the database server when initiating the connection,  This field is optional.
- `application_name` - The name to use in logging and analytics.  This field is optional.
- `port` - The port to connect to on the host. Default is `5432`.  This field is optional.
- `connect_timeout` - Sets the timeout applied to socket-level connection attempts. Default is no limit. This field is optional.

##### Running the server

To run the server

```sh
$ ./solana-accounts-proxy /path/to/Config.toml/file
```

To run the server with logging enabled pass one of `debug`, `info`, `trace`, `error` logging flags to `RUST_LOG=[flag]`. Example to log the RPC server requests and database queries use:

```sh
$ RUST_LOG=debug ./solana-accounts-proxy /path/to/ProxyConfig.toml/file
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


#### Compiling

To compile and run the crate

```sh
$ cargo run --release -- /path/to/ProxyConfig.toml/file
```

To compile and run the crate with logging enabled, pass one of the `debug`, `info`, `trace`, `error` log flags. An example to see the logging info of the RPC server and postgres queries, run:

```sh
$ RUST_LOG=debug cargo run --release -- /path/to/ProxyConfig.toml/file
```
