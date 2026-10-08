// Copyright (C) 2026 Parity Technologies (UK) Ltd.
// Copyright (C) 2026 Enjin
// SPDX-License-Identifier: GPL-3.0-or-later

use polkadot_rest_api_config::{ChainConfigs, ChainType, Hasher};

#[test]
fn ebc_network_configuration() {
    let configs = ChainConfigs::default();
    assert_eq!(configs.chain_names().len(), 4);
    for (spec, relay, fee_min) in [
        ("enjin", None, 100),
        ("canary", None, 100),
        ("matrix-enjin", Some("enjin"), 0),
        ("matrix", Some("canary"), 0),
    ] {
        let config = configs.get(spec).unwrap();
        assert_eq!(config.relay_chain.as_deref(), relay);
        assert_eq!(
            config.chain_type,
            if relay.is_some() {
                ChainType::Parachain
            } else {
                ChainType::Relay
            }
        );
        assert_eq!(config.para_id, relay.map(|_| 1000));
        assert_eq!(config.legacy_types, "none");
        assert_eq!(config.min_calc_fee_runtime, fee_min);
        assert_eq!(config.hasher, Hasher::Blake2_256);
        assert_eq!(config.block_number_bytes, 4);
        assert!(config.finalizes);
        assert!(configs.get(&spec.to_uppercase()).is_some());
    }
    for spec in [
        "polkadot",
        "kusama",
        "westend",
        "statemint",
        "asset-hub-polkadot",
        "unknown",
    ] {
        assert!(configs.get(spec).is_none());
    }
}
