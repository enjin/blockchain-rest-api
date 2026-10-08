# useRcBlock specification

On Matrixchain, useRcBlock interprets a block reference in the context of its
matching relay chain. Configure SAS_SUBSTRATE_MULTI_CHAIN_URL with a relay entry
before using relay-dependent operations. Enjin Matrixchain uses Enjin Relaychain;
Canary Matrixchain uses Canary Relaychain. The API rejects cross-network pairs.

Consult each endpoint's parameters for useRcBlock support. The /rc endpoints
query the relay directly. Historical lookups require the corresponding archive
state on both nodes. Matrixchain queries without a relay remain supported when
the selected operation does not require relay data.
