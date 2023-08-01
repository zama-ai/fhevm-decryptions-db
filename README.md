# fhEVM Decryptions Database

The fhEVM Decryptions Database is a key-value database of decryption results plus a signature for them from decryption oracles (or just oracles). It supports two operations:
* HTTP GET - executed by validators, full nodes or anyone in order to get a signed decryption result from the DB
* HTTP PUT - only executed by oracles in order to put a signed decryption result in the DB

Access control is left to an external HTTP service (e.g. a proxy) that sits in front of the DB. For example, one might allow GET requests from the Internet and only allow PUT request from oracles. Furthermore, TLS handling is also left to the external HTTP service.

Another point is that the intention for the database is to be as simple as possible, without trying to interpret the data it stores. For example, it only expects that the keys are 32 byte hashes and it doesn't impose anything on signatures. Rationale is that there is external access control such that only trusted oracles can write to the DB through HTTP PUT.

Currently, the DB doesn't support deletion of decryption results. That allows any node to catch up to the latest state from any previous point. If that approach proves problematic in terms of DB size and/or performance, we can consider pruning it in a future release.

## API
The DB exposes a REST API on the `/decryption/<key>` route. The `key` parameter is a hex-encoded byte buffer of 32 bytes (i.e. 64 characters in hex).
The DB supports the following methods.

### HTTP PUT
An oracle can put a decryption result to the DB via an HTTP PUT request with a JSON payload. For example:
```json
{
    "value": 42,
    "signature": "YmJiYg=="
}
```
The DB expects two fields:
* `value` - an uint64 decrypted value
* `signature` - a base64-encoded signature

Example request:
```bash
curl -v --header "Content-type: application/json" --request PUT \
  --data '{"value": 42, "signature": "YmJiYg=="}' \
  http://127.0.0.1:8001/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
```

### HTTP GET
Anyone can get a decrypted value via an HTTP GET request.

Example request:
```bash
curl -v http://127.0.0.1:8001/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
```

The resulting response has the same JSON format as for HTTP PUT:
```json
{
    "value": 42,
    "signature": "YmJiYg=="
}
```

## Note on Multiple Oracles
As of now, there could be multiple oracles putting decryptions on the DB. However, only a single signature is allowed per decryption. This behaviour will be changed in a future release.

## Note on Determinism
It is expected that oracles generate a single unique key for a particular ciphertext. For example, the key could be the hash of the ciphertext.

## Note on Signature
The DB doesn't impose anything on the signature field other than it being valid base64. It is up to the blockchain protocol to decide what piece of data is signed. For example, one might do `sign(hash(ciphertext) || value)`.

## Note on RocksDB
We use RocksDB as an underlying key-value store. We've chosen it, because it is battle-tested, performant, in-process, tweakable and supports concurrent calls from multiple threads. If needed, it can easily be replaced with another store.

## Note on Race Conditions Between Oracle and Non-Oracle Nodes
Since oracles are the only nodes that put decryption results into the database and since all nodes (oracle and non-oracle ones) execute smart contract code at the same time, there is a race condition between an oracle putting a result and any other node reading it. Currently, the solution to this problem is to use a `WaitCache` that keeps pending key-values in memory for a limited period of time. Additionally, it allows a get request to wait until the requested key is put by the oracle.

## Build and Run
### Local
```bash
cargo build --release
cargo run --release
```

### Docker
```bash
docker build -t fhevm-decryptions-db:latest .
docker run -d -p 8001:8001 fhevm-decryptions-db:latest
```

## Configuration
We use the Rocket-provided configuration file - [Rocket.toml](Rocket.toml). It supports a number of rocket-specific configuration options as documented here: https://rocket.rs/v0.5-rc/guide/configuration/#configuration

We've introduced the `testing` configuration profile that is only used for integration tests.

The following configuration options are currently supported:

`db_path` - A path to the RocksDB database.

`max_expected_oracle_delay_ms` - An HTTP GET might try to get a decryption that is not yet put by an oracle. This option configures the maximum time (in ms) that oracles are expected to be late with the put operation.

## Testing
Integration tests use a real RocksDB database. The database path is read from the `testing` profile in the configuration (Rocket.toml) file.

In order for tests to run properly, execute them one at a time, using the `testing` profile:
```bash
ROCKET_PROFILE=testing cargo test -- --test-threads=1
```
