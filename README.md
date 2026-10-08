# Polkadot REST API

A REST service for interacting with Polkadot SDK-based blockchain nodes, rewritten from the ground up in Rust.

## Enjin and Canary support

This fork adds core Relaychain and Matrixchain support for the following runtime spec names:

| Network | Spec name | Native token | Decimals | SS58 prefix |
|---------|-----------|--------------|----------|-------------|
| Enjin Relaychain | `enjin` | ENJ | 18 | 2135 |
| Canary Relaychain | `canary` | cENJ | 18 | 69 |
| Enjin Matrixchain | `matrix-enjin` | ENJ | 18 | 1110 |
| Canary Matrixchain | `matrix` | cENJ | 18 | 9030 |

Run one API instance per chain, pointing `SAS_SUBSTRATE_URL` at that
chain's RPC node. The runtime spec name selects the configuration automatically.
Balances, blocks and transactions on the primary chain need only that connection.

```bash
# Replace with your Relaychain or Matrixchain RPC URL.
export SAS_SUBSTRATE_URL=ws://127.0.0.1:9944
export SAS_EXPRESS_PORT=8080
cargo run --release --bin polkadot-rest-api
```

For Matrixchain, optionally add its matching relaychain to enable the existing
`/v1/rc/...` endpoints and parachain inclusion queries:

```bash
# Example: Matrixchain RPC on port 9944, its relaychain RPC on port 9945.
export SAS_SUBSTRATE_URL=ws://127.0.0.1:9944
export SAS_SUBSTRATE_MULTI_CHAIN_URL='[{"url":"ws://127.0.0.1:9945","type":"relay"}]'
export SAS_EXPRESS_PORT=8080
cargo run --release --bin polkadot-rest-api
```

Pair `matrix-enjin` with `enjin`, and `matrix` with `canary`. Both Matrixchains
use parachain ID 1000. Without a relay URL, primary-chain queries still work and
relay queries report that a relay connection is not configured. If a relay URL
is supplied, startup waits for that connection and fails if it cannot connect.
When switching back to a standalone instance, unset `SAS_SUBSTRATE_MULTI_CHAIN_URL`.

Matrixchains are classified as parachains. The Asset Hub-specific `useRcBlock`
parameter and Asset Hub migration endpoints are not supported; query Matrixchain
with its own block numbers/hashes and use `/v1/rc/...` for relaychain data.

The initial scope is native account balances and nonces, blocks and their decoded
extrinsics/events, runtime metadata, generic pallet storage, and the existing
transaction parsing, material, fee-estimation and submission endpoints. Balances
are returned in base units by default; `denominated=true` uses 18 decimals.
The existential deposit is read from the queried block's runtime metadata, with
0.1 ENJ/cENJ as the fallback. Address formatting prefers the node's `ss58Format`
property and falls back to the prefixes above.

Examples for the next live-network testing pass (all API paths start with `/v1`):

- `GET /v1/node/version` and `GET /v1/runtime/spec`
- `GET /v1/accounts/{accountId}/balance-info`
- `GET /v1/accounts/{accountId}/balance-info?denominated=true`
- `GET /v1/accounts/{accountId}/balance-info?at={blockHeight}`
- `GET /v1/blocks/head` and `GET /v1/blocks/{blockHeight}`
- `GET /v1/blocks/{blockHeight}/extrinsics/{extrinsicIndex}`
- `POST /v1/transaction/parse` and `POST /v1/transaction/fee-estimate`

Historical queries require a node retaining the requested block/state. Verify
funded and empty accounts, locked/reserved funds, transfers, batches, failed
extrinsics, and blocks around runtime upgrades against the existing Sidecar.
On Matrixchain, also check the optional relay connection and keep Matrixchain
and relaychain block heights distinct.
Events after the final extrinsic use upstream's `afterExtrinsics` field, rather
than the placeholder extrinsics used by the Enjin Sidecar workaround.

Live-network compatibility testing is a separate follow-up. This phase does not
adapt nomination pools or add Multi-Tokens, Fuel Tanks or collator-staking APIs.
Existing specialised upstream endpoints are not a guarantee
of compatibility with Enjin's custom pallets. Transaction access is by block and
extrinsic index; account transaction history and transaction-hash lookup require
an indexer.

## Public Instances

Parity hosts public instances of the Polkadot REST API for the following chains:

| Chain | URL |
|-------|-----|
| **Polkadot** | https://polkadot-relay-rest-api.parity.io/v1 |
| **Kusama** | https://kusama-relay-rest-api.parity.io/v1 |
| **Westend** | https://westend-relay-rest-api.parity.io/v1 |
| **Polkadot Asset Hub** | https://polkadot-hub-rest-api.parity.io/v1 |
| **Kusama Asset Hub** | https://kusama-hub-rest-api.parity.io/v1 |
| **Westend Asset Hub** | https://westend-hub-rest-api.parity.io/v1 |

Interactive API documentation is available at `/docs` on each instance (e.g., [Polkadot Docs](https://polkadot-relay-rest-api.parity.io/docs)).

**Usage Limits & Production Warning**

These public instances are subject to rate limiting and are not intended for production use. For consistent testing or development, it is highly recommended to implement a retry strategy (e.g., exponential backoff) to handle potential request throttling gracefully.

## Implementation Details

### Logging

Logging levels supported are ```trace, debug, info, http, warn, error```. **http** level allows for the emission of http information logging (method, route, elapsed time, success code). However currently tracing does not support *http*.  To mitigate this, **http** level falls back to *debug* for successful logs, *warn* for 4** request logs, and *error* for 5**

`SAS_LOG_LEVEL` is passed straight to `tracing_subscriber`'s `EnvFilter`, so it accepts full
directive strings and not only a bare level. That is how you reach the layers underneath the
handlers:

```bash
# transaction submissions: hash, byte length, elapsed time, and the reason for a rejection
export SAS_LOG_LEVEL=info

# add the RPC client, which is where a stalled or reconnecting connection shows up
export SAS_LOG_LEVEL="info,subxt_rpcs=debug"

# everything the RPC layer does, including individual requests and responses. Noisy.
export SAS_LOG_LEVEL="info,subxt=debug,subxt_rpcs=trace,jsonrpsee=trace"
```

Submissions to `POST /transaction` log twice at `info`, once on the way out and once with
the outcome, both carrying the extrinsic hash so a submission can be followed through to
the node. A rejection logs at `warn` with the reason, and an acceptance that took longer
than five seconds logs at `warn` rather than `info`.

Both lines are at the default level on purpose. If the RPC connection stalls, the outcome
line is never reached, so a `Submitting extrinsic` with no matching outcome for the same
hash is the signature of a stuck transaction. That only helps if it is visible without
raising the level first.

The extrinsic payload itself is only logged at `trace`. At every other level the lines
carry its hash and length and nothing more.

## Metrics and Monitoring

The API exposes Prometheus metrics at `/metrics`. To enable metrics collection, set:

```bash
export SAS_METRICS_ENABLED=true
```

A sample Grafana dashboard is provided in `metrics/grafana/provisioning/dashboards/` for visualizing metrics.

### docker compose

When running locally with `docker compose`, the Grafana dashboard is accessible at http://localhost:3000/d/polkadot-rest-api

If needed, the login and password for grafana are set to "admin" and "admin" respectfully.

All container resources are shown despite only the `rest-api` container being useful.
To map the short ids for a container name you can run

```bash
docker ps --format '{{.ID}}: {{.Names}}'
```

Prometheus is accessible at http://localhost:9090/

Loki logs can be viewed in Grafana at Explore > Loki (select Loki as the datasource and query with `{service_name="polkadot-rest-api"}`)

## Benchmarks

### Benchmark Workflows

The repository includes two main benchmark workflows that run automatically on pushes and pull requests to the `main` branch:

#### 1. Benchmark workflow

- Builds and starts the server
- Runs performance tests against all configured endpoints
- Measures throughput (requests/second) and latency metrics (P50, P90, P99)
- Publishes results to GitHub Pages for historical tracking

**GitHub Pages Dashboard**: [https://paritytech.github.io/polkadot-rest-api/dev/bench/](https://paritytech.github.io/polkadot-rest-api/dev/bench/)

#### 2. Benchmark Comparison (vs Sidecar)


- Runs benchmarks against public Sidecar instance
- Calculates performance differences and improvements
- Generates comparison reports with percentage differences
- Publishes comparison metrics to GitHub Pages for trend analysis

**GitHub Pages Dashboard**: [https://paritytech.github.io/polkadot-rest-api/dev/bench/comparison/](https://paritytech.github.io/polkadot-rest-api/dev/bench/comparison/)

### Benchmark Metrics

Both workflows track the following metrics:

- **Throughput**: Requests per second (higher is better)
- **Average Latency**: Mean response time in milliseconds (lower is better)
- **P50 Latency**: 50th percentile latency (lower is better)
- **P90 Latency**: 90th percentile latency (lower is better)
- **P99 Latency**: 99th percentile latency (lower is better)

## Testing

### Unit Tests

Unit tests are embedded in the source code and test individual functions and modules.

**Run all unit tests:**
```bash
cargo test --workspace --all-features
```

### Integration Tests

These tests are located in `crates/integration_tests/tests/`.

#### Available Integration Test Suites

1. **basic.rs** - Tests basic API endpoints (health, version)
2. **latest.rs** - Tests endpoints with the latest blockchain data
3. **historical.rs** - Tests endpoints with historical blockchain data

#### Test Configuration

Test definitions are located in `crates/integration_tests/tests/config/test_config.json`. To add new integration tests, add them to this configuration file.

#### Running Integration Tests

**Step 1:** Start the API server in one terminal

For Polkadot:
```bash
export SAS_SUBSTRATE_URL=wss://rpc.polkadot.io
cargo run --release --bin polkadot-rest-api
```

For Kusama:
```bash
export SAS_SUBSTRATE_URL=wss://kusama-rpc.polkadot.io
cargo run --release --bin polkadot-rest-api
```

**Step 2:** Run tests in another terminal

```bash
# Run all integration tests
cargo test --package integration_tests

# Run a specific test suite (recommended - cleaner output)
cargo test --package integration_tests --test historical  # All historical tests
cargo test --package integration_tests --test latest      # All latest tests
cargo test --package integration_tests --test basic       # All basic tests
```

**Running tests for a specific chain:**

```bash
# Historical tests (use fixtures for regression testing)
cargo test --package integration_tests --test historical test_historical_polkadot
cargo test --package integration_tests --test historical test_historical_kusama
cargo test --package integration_tests --test historical test_historical_asset_hub_polkadot
cargo test --package integration_tests --test historical test_historical_asset_hub_kusama

# Latest tests (test against live blockchain data)
cargo test --package integration_tests --test latest test_latest_polkadot
cargo test --package integration_tests --test latest test_latest_kusama
cargo test --package integration_tests --test latest test_latest_asset_hub_polkadot
cargo test --package integration_tests --test latest test_latest_asset_hub_kusama
```

**Viewing test output:**

By default, cargo captures test output. To see the detailed test progress with checkmarks and colored output, add `-- --nocapture`:

```bash
cargo test --package integration_tests --test historical test_historical_polkadot -- --nocapture
```

Example output:
```
Running 2 historical test cases for chain: polkadot

✓ /v1/blocks/{blockId} (block 1000000)
✓ /v1/blocks/{blockId} (block 10000000)

════════════════════════════════════════════════════════════
Historical Test Results for polkadot
════════════════════════════════════════════════════════════
  ✓ Passed: 2
  ✗ Failed: 0
════════════════════════════════════════════════════════════
```

### Updating Test Fixtures

To update test fixtures with current blockchain data:

```bash
./scripts/update_fixtures.sh
```

### Using `.env` files

The application supports configuration via `.env` files (for example, `.env` or `.env.polkadot`), allowing you to define all environment variables in one place. A sample configuration is available in [.env.sample](./.env.sample).
To start `polkadot-rest-api` with a specific `.env` file, pass the file path as a command-line argument:
```bash
cargo run --release --bin polkadot-rest-api -- --env-file .env.polkadot
```
This command loads the environment variables specified in `.env.polkadot` file, which should be located in the project's root directory.

### Multi-Chain Configuration

For Asset Hub deployments that need relay chain access (e.g., for `useRcBlock` parameter support or `/rc/` related endpoints), configure `SAS_SUBSTRATE_MULTI_CHAIN_URL`:

```bash
# Primary connection: Asset Hub
export SAS_SUBSTRATE_URL=wss://polkadot-asset-hub-rpc.polkadot.io

# Additional chain: Relay chain (enables useRcBlock)
export SAS_SUBSTRATE_MULTI_CHAIN_URL='[{"url":"wss://rpc.polkadot.io","type":"relay"}]'
```

**Supported chain types:**
- `relay` - Relay chain (Polkadot, Kusama, Westend, etc.)
- `assethub` - Asset Hub parachain
- `parachain` - Generic parachain
