# Enjin fork migration

The executable is now enjin-rest-api. The underlying Rust crates remain
polkadot-rest-api and polkadot-rest-api-config, including their Rust namespaces. Existing /v1 paths for retained endpoints and SAS_
settings remain compatible. Update service commands, container names, log
filters and metrics dashboards. The default metrics prefix is enjin_rest_api.

Only enjin, canary, matrix-enjin and matrix runtime specs are supported. The
API exposes native balances, blocks, extrinsics, runtime data and transaction
tools. It does not provide indexed account transaction history. Multi-Tokens
support is deferred. Asset Hub, Coretime and asset-specific upstream routes
are not registered. Pallet-specific APIs still require the relevant runtime
pallet; Matrixchain CollatorStaking is not exposed by the relay Staking API.

This modified fork retains Parity Technologies attribution and GPL-3.0-or-later.
See [fork notices](attribution.txt) and [license](license.txt).
