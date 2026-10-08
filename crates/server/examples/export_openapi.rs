// Copyright (C) 2026 Enjin
// SPDX-License-Identifier: GPL-3.0-or-later
use utoipa::OpenApi;
fn main() {
    println!(
        "{}",
        polkadot_rest_api::openapi::ApiDoc::openapi()
            .to_pretty_json()
            .unwrap()
    );
}
