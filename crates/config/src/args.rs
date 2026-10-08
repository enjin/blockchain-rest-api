// Copyright (C) 2026 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "enjin-rest-api", author, version, about = "Enjin Blockchain REST API", long_about = None)]
pub struct Args {
    /// Path to .env file (e.g., .env.enjin)
    #[arg(short, long, default_value = ".env")]
    pub env_file: String,
}

impl Args {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
