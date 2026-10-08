// Copyright (C) 2026 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::handlers::accounts;
use crate::routes::{API_VERSION, RegisterRoute, RouteRegistry};
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn accounts_routes(registry: &RouteRegistry) -> Router<AppState> {
    Router::new()
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/compare",
            "get",
            get(accounts::get_compare),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/balance-info",
            "get",
            get(accounts::get_balance_info),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/convert",
            "get",
            get(accounts::get_convert),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/proxy-info",
            "get",
            get(accounts::get_proxy_info),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/staking-info",
            "get",
            get(accounts::get_staking_info),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/staking-payouts",
            "get",
            get(accounts::get_staking_payouts),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/vesting-info",
            "get",
            get(accounts::get_vesting_info),
        )
        .route_registered(
            registry,
            API_VERSION,
            "/accounts/:accountId/validate",
            "get",
            get(accounts::get_validate),
        )
}
