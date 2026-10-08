# Enjin Blockchain REST API

REST access to Enjin Blockchain: native balances, blocks, extrinsics,
events, runtime metadata and storage, transaction parsing, fee estimation and
submission. This is an Enjin-maintained fork of
[Polkadot REST API](https://github.com/paritytech/polkadot-rest-api) by Parity
Technologies, licensed under GPL-3.0-or-later. See [NOTICE](NOTICE) and [LICENSE](LICENSE).

## Supported networks

| Network | Runtime spec | Token | Decimals | SS58 | Optional relay | RPC endpoint |
| --- | --- | --- | --- | --- | --- | --- |
| Enjin Relaychain | `enjin` | ENJ | 18 | 2135 | — | `wss://rpc.relay.blockchain.enjin.io` |
| Canary Relaychain | `canary` | cENJ | 18 | 69 | — | `wss://rpc.relay.canary.enjin.io` |
| Enjin Matrixchain | `matrix-enjin` | ENJ | 18 | 1110 | `enjin` | `wss://rpc.matrix.blockchain.enjin.io` |
| Canary Matrixchain | `matrix` | cENJ | 18 | 9030 | `canary` | `wss://rpc.matrix.canary.enjin.io` |

Other runtime specs fail at startup. Both Matrixchains use parachain ID 1000.
A configured relay must match the Matrixchain's network. Without a relay URL,
Matrixchain's own endpoints remain available; `/rc` operations need the relay.

Native balance and block/extrinsic queries are the initial focus. There is no
indexed account transaction-history endpoint: scan blocks or use an indexer.
Multi-Tokens and other APIs specific to Enjin Blockchain are deferred. Asset Hub, Coretime,
Assets, PoolAssets, ForeignAssets, NominationPools and AssetConversion routes
are not exposed. Other pallet-specific handlers depend on runtime metadata;
consult `/v1/capabilities` for the connected runtime. Historical upstream
implementations and fixtures remain for regression coverage, not as supported
networks. Matrixchain's CollatorStaking is not the relay Staking API.

## Build and run

Install Rust 1.94.0 (pinned in `rust-toolchain.toml`) and Node.js 22, then:

```sh
npm ci --prefix docs
npm run update-spec --prefix docs
npm run build --prefix docs
cargo build --locked --release -p polkadot-rest-api
SAS_SUBSTRATE_URL=wss://rpc.relay.blockchain.enjin.io \
SAS_SUBSTRATE_MULTI_CHAIN_URL='[]' \
SAS_EXPRESS_BIND_HOST=127.0.0.1 SAS_EXPRESS_PORT=8080 SAS_LOG_LEVEL=info \
./target/release/enjin-rest-api --env-file /dev/null
```

Alternatively, copy `.env.sample` to `.env` and run the binary. The local RPC
default remains `ws://127.0.0.1:9944`. `SAS_*` environment variables and `/v1`
paths are preserved. The executable is now `enjin-rest-api`; update your
service unit, command, or process-exporter configuration accordingly.

For Matrixchain, set its RPC URL and optionally set:

```sh
export SAS_SUBSTRATE_MULTI_CHAIN_URL='[{"url":"wss://rpc.relay.blockchain.enjin.io","type":"relay"}]'
```

Use a Canary relay URL for Canary Matrixchain. Supply an archive RPC when
querying historical state that a pruned node no longer retains.

## Test

```sh
curl --fail http://127.0.0.1:8080/v1/health
curl --fail http://127.0.0.1:8080/v1/blocks/head/header
curl --fail http://127.0.0.1:8080/v1/blocks/head
curl --fail http://127.0.0.1:8080/v1/accounts/ACCOUNT_ADDRESS/balance-info
python3 scripts/smoke-ebc.py --base-url http://127.0.0.1:8080 --spec enjin
```

Browse `/docs` for endpoint documentation and `/api-docs/openapi.json` for the
OpenAPI schema. Transactions are queried by block and extrinsic index. The
smoke test uses a read-only zero-account balance query; it never submits a
transaction. It accepts `--account` to check a funded account instead.

```sh
cargo fmt --all -- --check
cargo test --locked --workspace --all-features --exclude integration_tests
cargo test --locked -p integration_tests --test chain_config
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
```

The remaining integration-test suites contain upstream historical fixtures;
they are not part of CI. Run `scripts/smoke-ebc.py` locally against each network
for live checks, using `--relay-spec` to also check a Matrixchain relay connection.

## Containers and monitoring

```sh
docker compose up --build
```

Compose builds the API locally against Enjin Relaychain by default and binds
port 8080 to localhost. Override `SAS_SUBSTRATE_URL` and
`SAS_SUBSTRATE_MULTI_CHAIN_URL` for other Enjin Blockchain networks. The local monitoring
stack is available with `docker compose -f docker-compose.local.yml up --build`.

Published images will use `docker.io/enjin/blockchain-rest-api` once publishing
is configured. Metrics retain `/metrics` but their default prefix is now
`enjin_rest_api`; Loki uses service `enjin-rest-api`. The included
Grafana and Prometheus configurations use the new names. An explicitly set
`SAS_METRICS_PROMETHEUS_PREFIX` overrides the default.

The underlying Rust packages remain `polkadot-rest-api` and
`polkadot-rest-api-config`, with their original Rust namespaces. The executable
is named `enjin-rest-api`; the repository and container image are
`enjin/blockchain-rest-api`.

## Development and releases

The OpenAPI spec is generated offline from Rust annotations with
`npm run update-spec --prefix docs`. Build the docs before the final Rust build:
the binary embeds `docs/dist`. CI checks the committed JSON for drift and builds
the docs and release binary. Docker builds docs from source in its own stage.

See [RELEASE.md](RELEASE.md) for container publishing and corresponding-source
requirements. The inherited version
is 0.3.2; select a fork release version before creating the first release tag.
