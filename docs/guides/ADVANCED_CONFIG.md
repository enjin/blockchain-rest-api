# Enjin Blockchain configuration

Enjin Blockchain REST API connects to one Enjin Blockchain runtime per process. Supported specs
are enjin, canary, matrix-enjin and matrix. Other specs fail at startup.

| Setting | Default | Purpose |
| --- | --- | --- |
| SAS_SUBSTRATE_URL | ws://127.0.0.1:9944 | Primary Enjin Blockchain RPC |
| SAS_SUBSTRATE_MULTI_CHAIN_URL | empty | Optional matching relay RPC |
| SAS_EXPRESS_BIND_HOST | 127.0.0.1 | Listen address |
| SAS_EXPRESS_PORT | 8080 | HTTP port |
| SAS_LOG_LEVEL | info | Logging verbosity |
| SAS_METRICS_ENABLED | false | Enable metrics |
| SAS_METRICS_PROMETHEUS_PREFIX | enjin_rest_api | Metric names |

Run Enjin Relaychain:

```sh
SAS_SUBSTRATE_URL=wss://rpc.relay.blockchain.enjin.io SAS_SUBSTRATE_MULTI_CHAIN_URL='[]' ./target/release/enjin-rest-api --env-file /dev/null
```

For Matrixchain, set its primary RPC URL and optionally a matching relay:

```sh
export SAS_SUBSTRATE_MULTI_CHAIN_URL='[{"url":"wss://rpc.relay.blockchain.enjin.io","type":"relay"}]'
```

matrix-enjin requires enjin; matrix requires canary. A mismatch fails startup.
Relay proxy operations require a configured relay. Historical queries need an
archive RPC. Settings use the existing SAS_ prefix for deployment compatibility.
