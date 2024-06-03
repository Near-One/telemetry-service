//! Telemetry protocol data sent by NEAR clients.
//! Original source file: https://github.com/near/nearcore/blob/master/core/primitives/src/telemetry.rs

#[derive(serde::Deserialize, Debug)]
pub struct TelemetryAgentInfo {
    pub name: String,
    pub version: String,
    pub build: String,
    // Added in https://github.com/near/nearcore/pull/11444.
    pub protocol_version: Option<u32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TelemetrySystemInfo {
    pub bandwidth_download: u64,
    pub bandwidth_upload: u64,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub boot_time_seconds: i64,
}

#[derive(serde::Deserialize, Debug)]
pub struct TelemetryChainInfo {
    // Added in https://github.com/near/nearcore/pull/11444.
    pub chain_id: Option<String>,
    pub node_id: String,
    // Changed from `Option<AccountId>` to `Option<String>`.
    pub account_id: Option<String>,
    pub is_validator: bool,
    pub status: String,
    // Changed from `CryptoHash` to `String`.
    pub latest_block_hash: String,
    // Changed from `BlockHeight` to `u64`.
    pub latest_block_height: u64,
    pub num_peers: usize,
    pub block_production_tracking_delay: f64,
    pub min_block_production_delay: f64,
    pub max_block_production_delay: f64,
    pub max_block_wait_delay: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct TelemetryInfo {
    pub agent: TelemetryAgentInfo,
    pub system: TelemetrySystemInfo,
    pub chain: TelemetryChainInfo,
    // Extra telemetry information that will be ignored by the explorer frontend.
    pub extra_info: String,
}
