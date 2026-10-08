// Copyright (C) 2026 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::substrate::ChainType;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainConfigError {
    #[error("Failed to parse chain config JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Chain '{0}' not found in configuration")]
    ChainNotFound(String),
}

/// Hash function used by a chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hasher {
    #[default]
    Blake2_256,
    Keccak256,
}

impl<'de> Deserialize<'de> for Hasher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.parse().expect("invalid hasher string"))
    }
}

impl FromStr for Hasher {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hasher = match s.to_lowercase().replace('_', "-").as_str() {
            "keccak-256" | "keccak256" => Hasher::Keccak256,
            _ => Hasher::Blake2_256,
        };
        Ok(hasher)
    }
}

impl std::fmt::Display for Hasher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hasher::Blake2_256 => write!(f, "Blake2_256"),
            Hasher::Keccak256 => write!(f, "Keccak256"),
        }
    }
}

/// Chain-specific configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    #[serde(default = "default_finalizes")]
    pub finalizes: bool,

    #[serde(default)]
    pub min_calc_fee_runtime: u32,

    #[serde(default)]
    pub query_fee_details_unavailable_at: Option<u32>,

    #[serde(default)]
    pub query_fee_details_available_at: Option<u32>,

    #[serde(default = "default_block_number_bytes")]
    pub block_number_bytes: usize,

    #[serde(default)]
    pub hasher: Hasher,

    #[serde(default = "default_legacy_types")]
    pub legacy_types: String,

    #[serde(default)]
    pub spec_versions: Option<crate::SpecVersionChanges>,

    #[serde(default)]
    pub chain_type: ChainType,

    #[serde(default)]
    pub relay_chain: Option<String>,

    #[serde(default)]
    pub para_id: Option<u32>,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            finalizes: default_finalizes(),
            min_calc_fee_runtime: 0,
            query_fee_details_unavailable_at: None,
            query_fee_details_available_at: None,
            block_number_bytes: default_block_number_bytes(),
            hasher: Hasher::default(),
            legacy_types: default_legacy_types(),
            spec_versions: None,
            chain_type: ChainType::default(),
            relay_chain: None,
            para_id: None,
        }
    }
}

fn default_finalizes() -> bool {
    true
}

fn default_block_number_bytes() -> usize {
    4
}

fn default_legacy_types() -> String {
    "none".to_string()
}

/// QueryFeeDetails RPC availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryFeeDetailsStatus {
    Available,
    Unavailable,
    Unknown,
}

impl std::fmt::Display for QueryFeeDetailsStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryFeeDetailsStatus::Available => write!(f, "Available"),
            QueryFeeDetailsStatus::Unavailable => write!(f, "Unavailable"),
            QueryFeeDetailsStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ChainConfig {
    pub fn supports_fee_calculation(&self, spec_version: u32) -> bool {
        spec_version >= self.min_calc_fee_runtime
    }

    pub fn query_fee_details_status(&self, spec_version: u32) -> QueryFeeDetailsStatus {
        match (
            self.query_fee_details_unavailable_at,
            self.query_fee_details_available_at,
        ) {
            (Some(unavail), Some(avail)) => {
                if spec_version <= unavail {
                    QueryFeeDetailsStatus::Unavailable
                } else if spec_version >= avail {
                    QueryFeeDetailsStatus::Available
                } else {
                    QueryFeeDetailsStatus::Unknown
                }
            }
            (Some(unavail), None) => {
                if spec_version <= unavail {
                    QueryFeeDetailsStatus::Unavailable
                } else {
                    QueryFeeDetailsStatus::Unknown
                }
            }
            (None, Some(avail)) => {
                if spec_version >= avail {
                    QueryFeeDetailsStatus::Available
                } else {
                    QueryFeeDetailsStatus::Unknown
                }
            }
            (None, None) => QueryFeeDetailsStatus::Unknown,
        }
    }
}

/// Container for all chain configurations
#[derive(Debug, Clone)]
pub struct ChainConfigs {
    configs: HashMap<String, ChainConfig>,
}

impl Default for ChainConfigs {
    fn default() -> Self {
        Self::load_embedded()
    }
}

impl ChainConfigs {
    fn load_embedded() -> Self {
        const EMBEDDED_CONFIG: &str = include_str!("chain_config.json");
        Self::from_json_str(EMBEDDED_CONFIG).expect("Failed to parse embedded chain_config.json")
    }

    pub fn from_json_str(json: &str) -> Result<Self, ChainConfigError> {
        let raw: HashMap<String, ChainConfig> = serde_json::from_str(json)?;

        let configs: HashMap<String, ChainConfig> = raw
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        Ok(Self { configs })
    }

    pub fn get(&self, chain_name: &str) -> Option<&ChainConfig> {
        self.configs.get(&chain_name.to_lowercase())
    }

    /// Get all configured chain names
    pub fn chain_names(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_config_only_contains_ebc() {
        let configs = ChainConfigs::default();
        let mut names = configs.chain_names();
        names.sort();
        assert_eq!(names, vec!["canary", "enjin", "matrix", "matrix-enjin"]);
    }

    #[test]
    fn enjin_relay_configs_match_runtime_classification() {
        let configs = ChainConfigs::default();
        for name in ["enjin", "canary"] {
            let config = configs.get(name).expect("relay config");
            let chain_type = ChainType::from_spec_name(&name.to_uppercase());
            assert_eq!(chain_type, ChainType::Relay);
            assert_eq!(config.chain_type, chain_type);
            assert_eq!(chain_type.as_relay_chain(name).unwrap().spec_name(), name);
            assert!(config.finalizes);
            assert_eq!(config.block_number_bytes, 4);
            assert_eq!(config.hasher, Hasher::Blake2_256);
            // Enjin uses portable metadata, not Polkadot's pre-V14 type definitions.
            assert_eq!(config.legacy_types, "none");
            assert!(config.relay_chain.is_none());
            assert!(config.para_id.is_none());
            // Preserve the fee threshold from the Enjin Sidecar configuration.
            assert!(!config.supports_fee_calculation(99));
            assert!(config.supports_fee_calculation(100));
            // Detect RPC availability rather than importing Polkadot's version thresholds.
            assert_eq!(
                config.query_fee_details_status(100),
                QueryFeeDetailsStatus::Unknown
            );
        }
    }

    #[test]
    fn matrixchains_resolve_to_their_own_relay_without_asset_hub_behavior() {
        let configs = ChainConfigs::default();
        for (name, relay) in [("matrix-enjin", "enjin"), ("matrix", "canary")] {
            let config = configs.get(&name.to_uppercase()).expect("matrix config");
            assert_eq!(config.chain_type, ChainType::Parachain);
            assert_eq!(ChainType::from_spec_name(name), config.chain_type);
            assert_eq!(config.relay_chain.as_deref(), Some(relay));
            assert_eq!(config.para_id, Some(1000));
            assert_eq!(config.block_number_bytes, 4);
            assert_eq!(config.hasher, Hasher::Blake2_256);
            assert_eq!(config.legacy_types, "none");
            assert!(config.finalizes);
            // Matrixchain does not inherit Relaychain's Sidecar fee threshold.
            assert!(config.supports_fee_calculation(0));
            assert_eq!(
                config.query_fee_details_status(0),
                QueryFeeDetailsStatus::Unknown
            );
            let relay_config = configs.get(config.relay_chain.as_deref().unwrap()).unwrap();
            assert_eq!(relay_config.chain_type, ChainType::Relay);
            let standalone = crate::Config::single_chain(config.clone());
            assert!(!standalone.has_relay_chain());
            let paired = crate::Config::with_relay_chain(config.clone(), relay_config.clone());
            assert!(paired.has_relay_chain());
            assert_eq!(paired.rc.unwrap().chain_type, ChainType::Relay);
        }
    }

    #[test]
    fn test_hasher_from_str() {
        assert_eq!("blake2-256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("Blake2_256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("keccak-256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("keccak256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("unknown".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
    }

    #[test]
    fn test_chain_config_defaults() {
        let config = ChainConfig::default();
        assert!(config.finalizes);
        assert_eq!(config.min_calc_fee_runtime, 0);
        assert_eq!(config.block_number_bytes, 4);
        assert_eq!(config.hasher, Hasher::Blake2_256);
        assert_eq!(config.legacy_types, "none");
    }

    #[test]
    fn test_supports_fee_calculation() {
        let config = ChainConfig {
            min_calc_fee_runtime: 1000,
            ..Default::default()
        };

        assert!(!config.supports_fee_calculation(999));
        assert!(config.supports_fee_calculation(1000));
        assert!(config.supports_fee_calculation(1001));
    }

    #[test]
    fn test_query_fee_details_status() {
        let config = ChainConfig {
            query_fee_details_unavailable_at: Some(27),
            query_fee_details_available_at: Some(28),
            ..Default::default()
        };

        assert_eq!(
            config.query_fee_details_status(26),
            QueryFeeDetailsStatus::Unavailable
        );
        assert_eq!(
            config.query_fee_details_status(27),
            QueryFeeDetailsStatus::Unavailable
        );
        assert_eq!(
            config.query_fee_details_status(28),
            QueryFeeDetailsStatus::Available
        );
        assert_eq!(
            config.query_fee_details_status(29),
            QueryFeeDetailsStatus::Available
        );
    }

    #[test]
    fn test_chain_configs_from_json() {
        // Synthetic names isolate parsing behaviour from supported-network configuration.
        let json = r#"{
            "test-relay": {
                "finalizes": false,
                "minCalcFeeRuntime": 10,
                "queryFeeDetailsUnavailableAt": 20,
                "queryFeeDetailsAvailableAt": 30,
                "blockNumberBytes": 8,
                "hasher": "keccak-256",
                "legacyTypes": "test-legacy",
                "chainType": "relay"
            },
            "test-parachain": {
                "chainType": "parachain",
                "relayChain": "test-relay",
                "paraId": 2000
            }
        }"#;
        let configs = ChainConfigs::from_json_str(json).unwrap();
        let relay = configs.get("test-relay").unwrap();
        assert!(!relay.finalizes);
        assert_eq!(relay.min_calc_fee_runtime, 10);
        assert_eq!(relay.query_fee_details_unavailable_at, Some(20));
        assert_eq!(relay.query_fee_details_available_at, Some(30));
        assert_eq!(relay.block_number_bytes, 8);
        assert_eq!(relay.hasher, Hasher::Keccak256);
        assert_eq!(relay.legacy_types, "test-legacy");
        assert_eq!(relay.chain_type, ChainType::Relay);
        let parachain = configs.get("test-parachain").unwrap();
        assert_eq!(parachain.chain_type, ChainType::Parachain);
        assert_eq!(parachain.relay_chain.as_deref(), Some("test-relay"));
        assert_eq!(parachain.para_id, Some(2000));
    }

    #[test]
    fn test_chain_configs_case_insensitive_lookup() {
        let json = r#"{"Enjin": {"finalizes": true}}"#;
        let configs = ChainConfigs::from_json_str(json).unwrap();
        for name in ["Enjin", "enjin", "ENJIN"] {
            assert!(configs.get(name).is_some());
        }
    }

    #[test]
    fn test_hasher_display() {
        assert_eq!(Hasher::Blake2_256.to_string(), "Blake2_256");
        assert_eq!(Hasher::Keccak256.to_string(), "Keccak256");
    }

    #[test]
    fn test_hasher_debug() {
        assert_eq!(format!("{:?}", Hasher::Blake2_256), "Blake2_256");
        assert_eq!(format!("{:?}", Hasher::Keccak256), "Keccak256");
    }

    #[test]
    fn test_query_fee_details_status_display() {
        assert_eq!(QueryFeeDetailsStatus::Available.to_string(), "Available");
        assert_eq!(
            QueryFeeDetailsStatus::Unavailable.to_string(),
            "Unavailable"
        );
        assert_eq!(QueryFeeDetailsStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_query_fee_details_no_thresholds() {
        let config = ChainConfig::default();

        // With no thresholds set, should always be Unknown
        assert_eq!(
            config.query_fee_details_status(0),
            QueryFeeDetailsStatus::Unknown
        );
        assert_eq!(
            config.query_fee_details_status(1000),
            QueryFeeDetailsStatus::Unknown
        );
    }

    #[test]
    fn test_query_fee_details_only_unavailable_threshold() {
        let config = ChainConfig {
            query_fee_details_unavailable_at: Some(100),
            ..Default::default()
        };

        // Should be Unavailable before threshold, Unknown after
        assert_eq!(
            config.query_fee_details_status(99),
            QueryFeeDetailsStatus::Unavailable
        );
        assert_eq!(
            config.query_fee_details_status(100),
            QueryFeeDetailsStatus::Unavailable
        );
        assert_eq!(
            config.query_fee_details_status(101),
            QueryFeeDetailsStatus::Unknown
        );
    }

    #[test]
    fn test_query_fee_details_only_available_threshold() {
        let config = ChainConfig {
            query_fee_details_available_at: Some(100),
            ..Default::default()
        };

        // Should be Unknown before threshold, Available after
        assert_eq!(
            config.query_fee_details_status(99),
            QueryFeeDetailsStatus::Unknown
        );
        assert_eq!(
            config.query_fee_details_status(100),
            QueryFeeDetailsStatus::Available
        );
        assert_eq!(
            config.query_fee_details_status(101),
            QueryFeeDetailsStatus::Available
        );
    }

    #[test]
    fn test_query_fee_details_unavailable_equals_available() {
        // Edge case where both thresholds are the same value.
        // Expect inclusive semantics: the unavailable check (<=) takes precedence.
        let config = ChainConfig {
            query_fee_details_unavailable_at: Some(27),
            query_fee_details_available_at: Some(27),
            ..Default::default()
        };

        // Below the threshold -> Unavailable
        assert_eq!(
            config.query_fee_details_status(26),
            QueryFeeDetailsStatus::Unavailable
        );

        // At the threshold -> Unavailable due to inclusive semantics
        assert_eq!(
            config.query_fee_details_status(27),
            QueryFeeDetailsStatus::Unavailable
        );

        // Above the threshold -> Available
        assert_eq!(
            config.query_fee_details_status(28),
            QueryFeeDetailsStatus::Available
        );
    }

    #[test]
    fn test_supports_fee_calculation_at_zero() {
        let config = ChainConfig {
            min_calc_fee_runtime: 0,
            ..Default::default()
        };

        // Should support fee calculation from block 0
        assert!(config.supports_fee_calculation(0));
        assert!(config.supports_fee_calculation(1));
    }

    #[test]
    fn test_supports_fee_calculation_high_threshold() {
        let config = ChainConfig {
            min_calc_fee_runtime: 1_000_000,
            ..Default::default()
        };

        assert!(!config.supports_fee_calculation(999_999));
        assert!(config.supports_fee_calculation(1_000_000));
    }

    #[test]
    fn test_chain_config_clone() {
        let config1 = ChainConfig {
            finalizes: false,
            min_calc_fee_runtime: 123,
            query_fee_details_unavailable_at: Some(10),
            query_fee_details_available_at: Some(20),
            block_number_bytes: 8,
            hasher: Hasher::Keccak256,
            legacy_types: "custom".to_string(),
            spec_versions: Default::default(),
            chain_type: ChainType::Relay,
            relay_chain: None,
            para_id: None,
        };

        let config2 = config1.clone();
        assert_eq!(config1.finalizes, config2.finalizes);
        assert_eq!(config1.min_calc_fee_runtime, config2.min_calc_fee_runtime);
        assert_eq!(config1.hasher, config2.hasher);
        assert_eq!(config1.legacy_types, config2.legacy_types);
    }

    #[test]
    fn test_chain_configs_lookup_known_and_unknown() {
        let configs = ChainConfigs::default();

        // Existing chain
        assert!(configs.get("enjin").is_some());

        // Non-existing chain returns None
        assert!(configs.get("non-existent-chain").is_none());
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let invalid_json = r#"{ invalid json }"#;
        let result = ChainConfigs::from_json_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_json_object() {
        let empty_json = r#"{}"#;
        let configs = ChainConfigs::from_json_str(empty_json).unwrap();
        assert!(configs.get("enjin").is_none());
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let json = r#"{
            "test-chain": {
                "finalizes": false
            }
        }"#;

        let configs = ChainConfigs::from_json_str(json).unwrap();
        let config = configs.get("test-chain").unwrap();

        // Specified value
        assert!(!config.finalizes);

        // Should use defaults for unspecified fields
        assert_eq!(config.min_calc_fee_runtime, 0);
        assert_eq!(config.block_number_bytes, 4);
        assert_eq!(config.hasher, Hasher::Blake2_256);
        assert_eq!(config.legacy_types, "none");
    }

    #[test]
    fn test_hasher_variations() {
        // Test all case variations
        assert_eq!("blake2-256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("Blake2-256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("BLAKE2-256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("blake2_256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);
        assert_eq!("Blake2_256".parse::<Hasher>().unwrap(), Hasher::Blake2_256);

        assert_eq!("keccak-256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("Keccak-256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("KECCAK-256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("keccak256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
        assert_eq!("Keccak256".parse::<Hasher>().unwrap(), Hasher::Keccak256);
    }

    #[test]
    fn test_spec_versions_is_optional() {
        let json = r#"{"test": {"specVersions": null}}"#;
        let configs = ChainConfigs::from_json_str(json).unwrap();
        let config = configs.get("test").unwrap();
        assert!(config.spec_versions.is_none());
    }

    #[test]
    fn test_spec_versions_when_present() {
        let json = r#"{
            "test": {
                "specVersions": {"changes": {"0": 1000, "1000": 1001}}
            }
        }"#;
        let configs = ChainConfigs::from_json_str(json).unwrap();
        let config = configs.get("test").unwrap();
        assert!(config.spec_versions.is_some());
        let spec_versions = config.spec_versions.as_ref().unwrap();
        assert_eq!(spec_versions.get_version_at_block(500), Some(1000));
        assert_eq!(spec_versions.get_version_at_block(1000), Some(1001));
    }
}
